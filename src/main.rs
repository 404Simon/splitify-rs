#![recursion_limit = "1024"]

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use std::convert::Infallible;

    use axum::{
        extract::{Path, State},
        response::sse::{Event, KeepAlive, Sse},
        routing::get,
        Router,
    };
    use futures::stream::Stream;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use rustify_app::{
        app::*,
        db::init_db,
        features::{
            recurring_debts::handlers::scheduler::process_due_recurring_debts_internal,
            shopping_lists::{create_broadcaster, EventBroadcaster},
        },
    };
    use time::Duration;
    use tokio::sync::broadcast;
    use tokio_cron_scheduler::{Job, JobScheduler};
    use tokio_stream::{wrappers::BroadcastStream, StreamExt as _};
    use tower::ServiceBuilder;
    use tower_sessions::{Expiry, Session, SessionManagerLayer};
    use tower_sessions_sqlx_store::SqliteStore;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    if let Err(e) = dotenvy::dotenv() {
        // Silently ignore errors (file may not exist)
        eprintln!("Note: .env file not loaded: {}", e);
    }

    // Initialize structured logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // Default log levels: debug for our app, info for dependencies
                "rustify_app=debug,tower_http=debug,axum=info,sqlx=warn".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Rustify Splitify application");

    // Initialize database
    let pool = init_db()
        .await
        .expect("FATAL: Failed to initialize database - check DATABASE_URL and migrations");

    tracing::info!("Database initialized successfully");

    // Setup session store
    let session_store = SqliteStore::new(pool.clone());
    session_store
        .migrate()
        .await
        .expect("FATAL: Failed to migrate session store - database may be corrupted");

    tracing::debug!("Session store migrated successfully");

    // Session cookie configuration (configurable for development)
    // SESSION_SAME_SITE: "strict" (default), "lax", or "none"
    // SESSION_SECURE: "true" (default for production), "false" (for local HTTP
    // development)
    let same_site = std::env::var("SESSION_SAME_SITE")
        .unwrap_or_else(|_| "strict".to_string())
        .to_lowercase();
    let same_site = match same_site.as_str() {
        "lax" => tower_sessions::cookie::SameSite::Lax,
        "none" => tower_sessions::cookie::SameSite::None,
        _ => tower_sessions::cookie::SameSite::Strict, // Default: secure
    };

    let secure = std::env::var("SESSION_SECURE")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase()
        != "false";

    tracing::info!(
        same_site = ?same_site,
        secure = secure,
        "Configuring session cookies"
    );

    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::weeks(1))) // 7 days
        .with_same_site(same_site)
        .with_secure(secure);

    let conf = get_configuration(None).expect(
        "FATAL: Failed to load Leptos configuration - check Cargo.toml [package.metadata.leptos]",
    );
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    // Start recurring debts cron scheduler
    // Cron expression can be configured via RECURRING_DEBTS_CRON environment
    // variable Default: "0 0 6 * * *" (daily at 6:00 AM)
    // Format: sec min hour day_of_month month day_of_week
    let cron_expression =
        std::env::var("RECURRING_DEBTS_CRON").unwrap_or_else(|_| "0 0 6 * * *".to_string());

    tracing::info!(
        cron_expression = %cron_expression,
        "Setting up recurring debts scheduler"
    );

    let scheduler = JobScheduler::new()
        .await
        .expect("FATAL: Failed to create job scheduler - system resources may be exhausted");

    let pool_for_scheduler = pool.clone();
    let job = Job::new_async(cron_expression.as_str(), move |_uuid, _lock| {
        let pool_clone = pool_for_scheduler.clone();
        Box::pin(async move {
            tracing::info!("Running scheduled recurring debts generation");

            // Call the internal function directly - no need for provide_context
            match process_due_recurring_debts_internal(pool_clone).await {
                Ok(count) => {
                    tracing::info!(count = count, "Successfully generated recurring debts");
                }
                Err(e) => {
                    tracing::error!(
                        error = %e,
                        "Failed to process recurring debts"
                    );
                }
            }
        })
    })
    .expect("FATAL: Failed to create cron job - check RECURRING_DEBTS_CRON syntax");

    scheduler
        .add(job)
        .await
        .expect("FATAL: Failed to add job to scheduler");

    scheduler
        .start()
        .await
        .expect("FATAL: Failed to start scheduler");

    tracing::info!("Recurring debts scheduler started successfully");

    // Create event broadcaster for shopping list real-time updates
    let broadcaster = create_broadcaster();
    tracing::debug!("Shopping list event broadcaster created");

    // SSE endpoint handler for shopping list updates
    async fn shopping_list_events(
        Path((_group_id, list_id)): Path<(i64, i64)>,
        State(broadcaster): State<EventBroadcaster>,
        _session: Session,
    ) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
        // Get or create channel for this list
        let tx = {
            let mut map = broadcaster.write();
            map.entry(list_id)
                .or_insert_with(|| broadcast::channel(100).0)
                .clone()
        };

        let rx = tx.subscribe();
        let stream = BroadcastStream::new(rx).filter_map(|result| match result {
            Ok(event) => {
                // Serialize event to JSON
                match serde_json::to_string(&event) {
                    Ok(json) => Some(Ok(Event::default().data(json))),
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to serialize SSE event");
                        None
                    }
                }
            }
            Err(e) => {
                tracing::debug!(error = %e, "SSE broadcast stream error");
                None
            }
        });

        Sse::new(stream).keep_alive(KeepAlive::default())
    }

    let sse_router = Router::new()
        .route(
            "/api/groups/{group_id}/shopping-lists/{list_id}/events",
            get(shopping_list_events),
        )
        .with_state(broadcaster.clone());

    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            {
                let leptos_options = leptos_options.clone();
                let pool = pool.clone();
                let broadcaster = broadcaster.clone();
                move || {
                    provide_context(leptos_options.clone());
                    provide_context(pool.clone());
                    provide_context(broadcaster.clone());
                }
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .merge(sse_router)
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(ServiceBuilder::new().layer(session_layer))
        .with_state(leptos_options)
        .with_state(broadcaster);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    tracing::info!("Server listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("FATAL: Failed to bind to address - port may already be in use");
    axum::serve(listener, app.into_make_service())
        .await
        .expect("FATAL: Server error during runtime");
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}

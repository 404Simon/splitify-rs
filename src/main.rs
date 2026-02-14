#![recursion_limit = "1024"]

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use rustify_app::app::*;
    use rustify_app::db::init_db;
    use rustify_app::features::recurring_debts::handlers::process_due_recurring_debts;
    use time::Duration;
    use tokio_cron_scheduler::{Job, JobScheduler};
    use tower::ServiceBuilder;
    use tower_sessions::{Expiry, SessionManagerLayer};
    use tower_sessions_sqlx_store::SqliteStore;

    // Initialize database
    let pool = init_db()
        .await
        .expect("FATAL: Failed to initialize database - check DATABASE_URL and migrations");

    // Setup session store
    let session_store = SqliteStore::new(pool.clone());
    session_store
        .migrate()
        .await
        .expect("FATAL: Failed to migrate session store - database may be corrupted");

    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::weeks(1))); // 7 days

    let conf = get_configuration(None).expect(
        "FATAL: Failed to load Leptos configuration - check Cargo.toml [package.metadata.leptos]",
    );
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    // Start recurring debts cron scheduler
    // Cron expression can be configured via RECURRING_DEBTS_CRON environment variable
    // Default: "0 0 6 * * *" (daily at 6:00 AM)
    // Format: sec min hour day_of_month month day_of_week
    let cron_expression =
        std::env::var("RECURRING_DEBTS_CRON").unwrap_or_else(|_| "0 0 6 * * *".to_string());

    log!(
        "Setting up recurring debts scheduler with cron expression: {}",
        cron_expression
    );

    let scheduler = JobScheduler::new()
        .await
        .expect("FATAL: Failed to create job scheduler - system resources may be exhausted");

    let pool_for_scheduler = pool.clone();
    let job = Job::new_async(cron_expression.as_str(), move |_uuid, _lock| {
        let pool_clone = pool_for_scheduler.clone();
        Box::pin(async move {
            log!("Running scheduled recurring debts generation...");

            // Provide the pool context for the server function
            provide_context(pool_clone.clone());

            match process_due_recurring_debts().await {
                Ok(count) => {
                    log!("Successfully generated {} recurring debts", count);
                }
                Err(e) => {
                    eprintln!("Error processing recurring debts: {}", e);
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

    log!("Recurring debts scheduler started successfully");

    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            {
                let leptos_options = leptos_options.clone();
                let pool = pool.clone();
                move || {
                    provide_context(leptos_options.clone());
                    provide_context(pool.clone());
                }
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(ServiceBuilder::new().layer(session_layer))
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
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

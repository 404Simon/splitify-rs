#[cfg(feature = "ssr")]
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

/// Initialize database connection pool and run migrations
#[cfg(feature = "ssr")]
pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:splitify.db".to_string());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

#[cfg(feature = "ssr")]
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

/// Initialize database connection pool and run migrations
#[cfg(feature = "ssr")]
pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:splitify.db".to_string());

    // Calculate connection pool size based on CPU cores
    // Formula: (cores * 2) + effective_spindle_count
    // For SQLite (single spindle), we use: cores * 2 + 1
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4); // Default to 4 cores if detection fails
    let max_connections = (cores * 2 + 1).clamp(5, 20); // Between 5 and 20

    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections as u32)
        .connect(&database_url)
        .await?;

    // Enable SQLite performance optimizations
    sqlx::query("PRAGMA journal_mode = WAL")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA synchronous = NORMAL")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA cache_size = -64000") // 64MB cache
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA busy_timeout = 5000") // 5 second timeout
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

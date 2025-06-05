use std::str::FromStr;

#[cfg(feature = "sqlite")]
pub use sqlx::SqlitePool as DbPool;

#[cfg(feature = "postgres")]
pub use sqlx::PgPool as DbPool;

#[cfg(feature = "sqlite")]
pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    let options = sqlx::sqlite::SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(5));
    DbPool::connect_with(options).await
}

#[cfg(feature = "postgres")]
pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

    let options = PgConnectOptions::from_str(database_url)?.application_name("webidler");

    PgPoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
}

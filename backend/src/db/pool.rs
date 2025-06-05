use sqlx::ConnectOptions;
use std::time::Duration;

#[cfg(feature = "sqlite")]
pub use sqlx::SqlitePool as DbPool;

#[cfg(feature = "postgres")]
pub use sqlx::PgPool as DbPool;

#[cfg(feature = "sqlite")]
pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(database_url)
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));
    DbPool::connect_with(options).await
}

#[cfg(feature = "postgres")]
pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    let options = sqlx::postgres::PgConnectOptions::from_str(database_url)
        .max_connections(5)
        .unwrap()
        .application_name("webidler");
    DbPool::connect_with(options).await
}

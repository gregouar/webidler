use std::net::SocketAddr;

use axum::{
    routing::{any, get},
    Router,
};
use http::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use tokio::signal;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use backend::{
    app_state::{AppSettings, AppState},
    db::{self, pool},
    email::EmailService,
    game::{
        data::master_store::MasterStore, sessions::SessionsStore, systems::sessions_controller,
    },
    rest, tasks, websocket,
};

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let master_store = MasterStore::load_from_folder("data")
        .await
        .expect("couldn't load master game data");

    // TODO: depending on environment, only install necessary
    sqlx::any::install_default_drivers();

    let db_pool =
        pool::create_pool(&std::env::var("DATABASE_URL").expect("missing 'DATABASE_URL' setting"))
            .await
            .expect("failed to connect to database");

    pool::migrate(&db_pool)
        .await
        .expect("failed to migrate database");

    migrate_data(&db_pool, &master_store)
        .await
        .expect("failed to migrate data");

    db::game_sessions::clean_all_sessions(&db_pool)
        .await
        .expect("couldn't clean game sessions");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let tracer_layer =
        TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true));

    let cors_layer = CorsLayer::new()
        .allow_origin([std::env::var("CORS_FRONTEND_URL")
            .expect("missing 'CORS_FRONTEND_URL' setting")
            .parse::<HeaderValue>()
            .unwrap()])
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION]);

    let sessions_store = SessionsStore::new();

    let purge_sessions_handle = tokio::spawn(tasks::purge_sessions(
        db_pool.clone(),
        sessions_store.clone(),
    ));

    let app_state = AppState {
        app_settings: AppSettings::from_env(),
        db_pool: db_pool.clone(),
        email_service: EmailService::from_env(),
        master_store,
        sessions_store: sessions_store.clone(),
    };

    let app = Router::new()
        .route("/", get(|| async { "OK" }))
        .merge(rest::routes(app_state.clone()))
        .route("/ws", any(websocket::handler))
        .with_state(app_state.clone())
        .layer(tracer_layer)
        .layer(cors_layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4200").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    let server = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    );

    tokio::select! {
        _ = server => {},
        _ = signal::ctrl_c() => {
            println!("Received shutdown signal");
        }
    }

    purge_sessions_handle.abort();

    // Note that this only save the sessions that were not active but in the store...
    if let Err(e) = sessions_controller::save_all_sessions(&db_pool, &sessions_store).await {
        tracing::error!("failed to save all sessions: {}", e);
    }

    tracing::debug!("server has been shut down");
}

async fn migrate_data(db_pool: &db::DbPool, master_store: &MasterStore) -> anyhow::Result<()> {
    db::migrations::migration_0_1_0_to_0_1_1::migrate(db_pool, master_store).await?;
    db::migrations::migration_0_1_1_to_0_1_2::migrate(db_pool).await?;
    db::migrations::migration_0_1_2_to_0_1_3::migrate(db_pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_master_store() {
        MasterStore::load_from_folder("../data").await.unwrap();
    }
}

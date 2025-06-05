use std::net::SocketAddr;

use axum::{
    extract::{FromRef, State},
    response::Json,
    routing::{any, get},
    Router,
};
use dotenvy;
use http::{HeaderValue, Method};
use tokio::signal;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use shared::http::server::PlayersCountResponse;

use backend::{
    db::pool::{self, DbPool},
    game::{data::master_store::MasterStore, session::SessionsStore},
    tasks, ws_connect,
};

#[derive(Debug, Clone)]
struct AppState {
    db_pool: DbPool,
    master_store: MasterStore,
    sessions_store: SessionsStore,
}
impl FromRef<AppState> for DbPool {
    fn from_ref(app_state: &AppState) -> DbPool {
        app_state.db_pool.clone()
    }
}
impl FromRef<AppState> for MasterStore {
    fn from_ref(app_state: &AppState) -> MasterStore {
        app_state.master_store.clone()
    }
}
impl FromRef<AppState> for SessionsStore {
    fn from_ref(app_state: &AppState) -> SessionsStore {
        app_state.sessions_store.clone()
    }
}

#[tokio::main]
async fn main() {
    // enable logging, since log defaults to silent
    std::env::set_var("RUST_LOG", "debug");

    let _ = dotenvy::dotenv();

    // TODO: depending on environment, only install necessary
    sqlx::any::install_default_drivers();

    let db_pool =
        pool::create_pool(&std::env::var("DATABASE_URL").expect("missing 'DATABASE_URL' setting"))
            .await
            .expect("failed to connect to database");

    sqlx::migrate!()
        .run(&db_pool)
        .await
        .expect("failed to migrate database");

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
        .allow_origin([
            "http://127.0.0.1:8080".parse::<HeaderValue>().unwrap(),
            "https://gregouar.github.io".parse::<HeaderValue>().unwrap(),
        ])
        .allow_methods([Method::GET, Method::POST]);

    let master_store = MasterStore::load_from_folder("data")
        .await
        .expect("couldn't load master game data");

    let sessions_store = SessionsStore::new();

    let purge_sessions_handle = tokio::spawn(tasks::purge_sessions(sessions_store.clone()));

    let app = Router::new()
        .route("/", get(|| async { "OK" }))
        .route("/players", get(get_players_count))
        .route("/ws", any(ws_connect::ws_handler))
        .with_state(AppState {
            db_pool,
            master_store,
            sessions_store,
        })
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
}

async fn get_players_count(
    State(sessions_store): State<SessionsStore>,
) -> Json<PlayersCountResponse> {
    Json(PlayersCountResponse {
        value: *sessions_store.players.lock().unwrap(),
    })
}

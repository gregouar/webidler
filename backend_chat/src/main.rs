use std::net::SocketAddr;

use axum::{
    Router,
    routing::{any, get},
};
use backend_shared::profanities_checker::ProfanitiesChecker;
use http::{
    HeaderValue, Method,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use tokio::signal;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use backend_chat::{
    app_state::{AppSettings, AppState},
    chat::messages_processor::MessagesProcessor,
    rest, websocket,
};

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let default_level = if cfg!(debug_assertions) {
        format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME"))
    } else {
        format!("{}=info,tower_http=info", env!("CARGO_CRATE_NAME"))
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| default_level.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let tracer_layer =
        TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true));

    let cors_origins = std::env::var("CORS_ORIGINS")
        .expect("missing 'CORS_ORIGINS'")
        .split(',')
        .map(|s| s.trim().parse::<HeaderValue>().unwrap())
        .collect::<Vec<_>>();

    let cors_layer = CorsLayer::new()
        .allow_origin(cors_origins)
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION]);

    let messages_processor = MessagesProcessor::new(
        ProfanitiesChecker::load_from_file(
            "profanities/strong_profanities.txt",
            "profanities/weak_profanities.txt",
        )
        .expect("failed to load profanities"),
    );

    let app_state = AppState {
        app_settings: AppSettings::from_env(),
        chat_state: messages_processor.get_chat_state(),
    };

    let messages_processor_handle = tokio::spawn(MessagesProcessor::run(messages_processor));

    let app = Router::new()
        .route("/", get(|| async { "OK" }))
        .merge(rest::routes())
        .route("/ws", any(websocket::handler))
        .with_state(app_state.clone())
        .layer(tracer_layer)
        .layer(cors_layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4242").await.unwrap();
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

    messages_processor_handle.abort();

    tracing::debug!("server has been shut down");
}

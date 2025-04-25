use std::net::SocketAddr;

use axum::{
    response::Json,
    routing::{any, get},
    Router,
};

use http::{HeaderValue, Method};
use shared::data::world::{HelloSchema, OtherSchema};
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use backend::ws_connect;

#[tokio::main]
async fn main() {
    // enable logging, since log defaults to silent
    std::env::set_var("RUST_LOG", "debug");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cors_layer = CorsLayer::new()
        .allow_origin([
            "http://127.0.0.1:8080".parse::<HeaderValue>().unwrap(),
            "https://gregouar.github.io".parse::<HeaderValue>().unwrap(),
        ])
        .allow_methods([Method::GET, Method::POST]);

    let app = Router::new()
        .route("/", get(|| async { "OK" }))
        .route("/hello", get(get_hello))
        .route("/other", get(get_other))
        .route("/ws", any(ws_connect::ws_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .layer(cors_layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4200").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

async fn get_hello() -> Json<HelloSchema> {
    Json(HelloSchema {
        greeting: String::from("hello pou"),
        value: 42,
    })
}

async fn get_other() -> Json<OtherSchema> {
    Json(OtherSchema {
        other: String::from("other pou"),
        value: 42,
    })
}

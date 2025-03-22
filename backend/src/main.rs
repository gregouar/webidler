use axum::{response::Json, routing::get, Router};

use http::{HeaderValue, Method};
use shared::data::{HelloSchema, OtherSchema};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let cors_layer = CorsLayer::new()
        .allow_origin("http://127.0.0.1:8080".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST]);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/hello", get(get_hello))
        .route("/other", get(get_other))
        .layer(ServiceBuilder::new().layer(cors_layer));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4200").await.unwrap();
    axum::serve(listener, app).await.unwrap();
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

use axum::{response::Json, routing::get, Router};
use serde_json::{json, Value};

use http::Method;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let cors_layer = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST]);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/pou", get(get_test))
        .route("/jsonpou", get(get_json))
        .layer(ServiceBuilder::new().layer(cors_layer));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4200").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_test() -> &'static str {
    "poupou".into()
}

async fn get_json() -> Json<Value> {
    Json(json!({ "data": 42 }))
}

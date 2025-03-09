use axum::{response::Json, routing::get, Router};
use serde_json::{json, Value};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/pou", get(get_test))
        .route("/jsonpou", get(get_json));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4200").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_test() -> &'static str {
    "poupou".into()
}

async fn get_json() -> Json<Value> {
    Json(json!({ "data": 42 }))
}

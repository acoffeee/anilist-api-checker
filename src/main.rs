use axum::{
    response::{Html, Json},
    routing::get,
    Router,
};

mod poller;
mod database;
mod routes;
use poller::poller;
use routes::{latency::list, latency_latest::latest};
#[tokio::main]
async fn main() {
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".into());
    let db = start_db();
    tokio::spawn(poller(db.clone()));

    let app = Router::new()
        .route("/", get(|| async { Html(include_str!("../index.html")) }))
        .route("/latency", get(list))
        .route("/latency/latest", get(latest))
        .with_state(db);

    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
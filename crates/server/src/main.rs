use axum::{Router, response::Html, routing::get};

mod database;
mod poller;
mod routes;
mod types;

use database::start_db;
use poller::poller;
use routes::{pings_latest::pings_latest, pings_with_limit::pings_with_limit};

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".into());
    let db = start_db();
    tokio::spawn(poller(db.clone()));

    let app = Router::new()
        .route("/", get(|| async { Html(include_str!("../index.html")) }))
        .route("/pings", get(pings_with_limit))
        .route("/pings/latest", get(pings_latest))
        .with_state(db);

    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

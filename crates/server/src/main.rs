use axum::{
    Router, middleware,
    routing::{get, post},
};

mod api;
mod database;
mod types;
use crate::api::routes::{
    private::insert_ping::insert_ping,
    public::{pings_latest::pings_latest, pings_with_limit::pings_with_limit, root::root},
};
use database::start_db;
use tokio::net::TcpListener;
use types::AppState;
use dotenv::dotenv;
#[tokio::main]
async fn main() {
    dotenv().ok();
    let port = match std::env::var("PORT") {
        Ok(port) => port,
        Err(_) => "3000".into(),
    };
    let api_keys: Vec<String> = std::env::var("API_KEYS")
        .expect("API_KEYS env var must be set")
        .split(',')
        .map(|k| k.trim().to_string())
        .filter(|k| !k.is_empty())
        .collect();
    let api_keys = AppState {
        api_keys: std::sync::Arc::new(api_keys),
    };
    let db = start_db().await;

    let app = Router::new()
        .route("/", get(root))
        .route("/pings", get(pings_with_limit))
        .route("/pings/latest", get(pings_latest))
        .route("/private/insert_ping", post(insert_ping))
        .route_layer(middleware::from_fn_with_state(
            api_keys.clone(),
            api::middleware::require_api_key,
        ))
        .with_state(db);

    let listener = TcpListener::bind(&format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

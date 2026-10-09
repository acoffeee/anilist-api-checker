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
use types::{AppState, Pokers};
use dotenv::dotenv;
mod Events;
use std::sync::Arc;
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
    let db = start_db().await;
    let event_list = Pokers {
        aws_lambda: false
    };
    let state = AppState {
        api_keys: Arc::new(api_keys),
        db,
        pokers: Arc::new(event_list),
    };
    
    let public_endpoints = Router::new()
        .route("/pings", get(pings_with_limit))
        .route("/pings/latest", get(pings_latest));
    
    let private_apis = Router::new()
        .route("/insert_ping", post(insert_ping))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            api::middleware::require_api_key,
        ));
    
    let app = Router::new()
        .route("/", get(root))
        .nest("/public", public_endpoints)
        .nest("/private/api", private_apis)
        .with_state(state);
    let listener = TcpListener::bind(&format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

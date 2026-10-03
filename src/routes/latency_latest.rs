use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use std::sync::{Arc, Mutex};
use crate::database::{Db, fetch_latest_rows, Row};
// GET /latency/latest
async fn latest(State(db): State<Db>) -> Result<Json<Row>, StatusCode> {
    let conn = db.lock().unwrap();
    fetch_latest_rows(&conn, 1)
        .map(Json)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })
}

use crate::database::fetch_latest_row;
use crate::types::{Db, Row};
use axum::{extract::State, http::StatusCode, response::Json};

// GET /pings/latest
pub async fn pings_latest(State(db): State<Db>) -> Result<Json<Row>, StatusCode> {
    let conn = db.lock().unwrap();
    fetch_latest_row(&conn).map(Json).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })
}

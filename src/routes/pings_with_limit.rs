use crate::database::fetch_latest_rows;
use crate::types::{Db, ListParams, Row};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};

// GET /pings?limit=60
pub async fn pings_with_limit(
    State(db): State<Db>,
    Query(p): Query<ListParams>,
) -> Result<Json<Vec<Row>>, StatusCode> {
    let limit = p.limit.unwrap_or(60).min(43200);
    let conn = db.lock().unwrap();
    fetch_latest_rows(&conn, limit)
        .map(Json)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })
}

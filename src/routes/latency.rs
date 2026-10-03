use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use std::sync::{Arc, Mutex};
use crate::database::{Db, row_from, Row};

// GET /latency?limit=60
async fn list(
    State(db): State<Db>,
    Query(p): Query<ListParams>,
) -> Result<Json<Vec<Row>>, StatusCode> {
    let limit = p.limit.unwrap_or(60).min(43200);
    let conn = db.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT ts, ok, status, latency_ms FROM pings ORDER BY ts DESC LIMIT ?1")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rows = stmt
        .query_map(params![limit], row_from)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows))
}

use crate::database;
use crate::types::{Db, PingsWithLimitParams, Ping};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
/// GET /pings_with_limit/limit=100
pub async fn pings_with_limit(
    State(db): State<Db>,
    Query(params): Query<PingsWithLimitParams>,
) -> Result<Json<Vec<Ping>>, StatusCode> {
    let limit = params.limit.unwrap_or(100).min(1000);
    database::fetch_latest_pings(&db, limit)
        .await
        .map(Json)
        .map_err(|e| {
            eprintln!("{e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

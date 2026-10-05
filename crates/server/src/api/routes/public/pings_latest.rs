use axum::{Json, extract::State, http::StatusCode};

use crate::database;
use crate::types::{Db, Row};
/// GET /pings/latest
pub async fn pings_latest(State(db): State<Db>) -> Result<Json<Row>, StatusCode> {
    match database::fetch_latest_row(&db).await {
        Ok(Some(row)) => Ok(Json(row)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("{e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

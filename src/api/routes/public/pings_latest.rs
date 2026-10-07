use axum::{Json, extract::State, http::StatusCode};

use crate::database;
use crate::types::{AppState, Ping};
/// GET /pings/latest
pub async fn pings_latest(state: State<AppState>) -> Result<Json<Ping>, StatusCode> {
    let db = state.db.clone();
    match database::fetch_latest_ping(&db).await {
        Ok(Some(Ping)) => Ok(Json(Ping)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("{e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

use axum::{Json, response::IntoResponse};
pub async fn insert_ping() -> impl IntoResponse {
    Json(serde_json::json!({"message": "Ping inserted successfully"}))
}

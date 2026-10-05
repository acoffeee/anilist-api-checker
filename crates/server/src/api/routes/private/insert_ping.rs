use axum::{response::IntoResponse, Json};
pub async fn insert_ping() -> impl IntoResponse {
    Json(serde_json::json!({"message": "Ping inserted successfully"}))
}
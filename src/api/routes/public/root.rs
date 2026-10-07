use axum::{Router, routing::get};
pub async fn root() -> impl axum::response::IntoResponse {
    axum::response::Html(include_str!("../../../../index.html"))
}

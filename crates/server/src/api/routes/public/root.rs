use axum::{routing::get, Router};
pub async fn root() -> impl axum::response::IntoResponse {
    axum::response::Html(include_str!("../../../../../../index.html"))
}
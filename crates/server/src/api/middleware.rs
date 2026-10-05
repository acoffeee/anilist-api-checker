use axum::{
    extract::{State, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use crate::types::AppState;
use subtle::ConstantTimeEq;
pub async fn require_api_key(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let provided = req
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
 
    // Constant-time comparison against every key (no short-circuit),
    // so response timing doesn't leak how much of a key matched.
    let valid = state.api_keys.iter().fold(false, |acc, key| {
        acc | bool::from(key.as_bytes().ct_eq(provided.as_bytes()))
    });
 
    if valid {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
 

use std::sync::Arc;
use tokio_rusqlite::{Connection, Error};
use serde::{Deserialize, Serialize};
use tokio_rusqlite::rusqlite;
#[derive(Serialize)]
pub struct Row {
    pub ts: i64,
    pub ok: bool,
    pub status: u16,
    pub latency_ms: i64,
}

#[derive(Deserialize)]
pub struct PingsWithLimitParams {
    pub limit: Option<u32>,
}
#[derive(Debug, Clone)]
pub struct PingResult {
    pub ok: bool,
    pub status: u16, // 0 = no response (timeout, DNS, connection error)
    pub latency_ms: i64,
}
#[derive(Debug, Clone)]
pub struct AppState {
    pub api_keys: Arc<Vec<String>  >,
}
//ig its cheap to copy so we can just clone it instead of wrapping it in an Arc
pub type Db = tokio_rusqlite::Connection;
pub type DbResult<T> = Result<T, tokio_rusqlite::Error<rusqlite::Error>>;

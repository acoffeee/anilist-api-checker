use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_rusqlite::rusqlite;
use tokio_rusqlite::{Connection, Error};
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ping {
    pub ok: bool,
    pub status: u16, // 0 = no response (timeout, DNS, connection error)
    pub latency_ms: i64,
    pub region: String,
    pub time: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct responsePing {
    pub success: bool,
    //a vps would use this to determine when to ping again, but faas doesnt use this
    pub time_to_next_ping: i64
}
#[derive(Debug, Clone)]
pub struct AppState {
    pub api_keys: Arc<Vec<String>>,
}
//ig its cheap to copy so we can just clone it instead of wrapping it in an Arc
pub type Db = tokio_rusqlite::Connection;
pub type DbResult<T> = Result<T, tokio_rusqlite::Error<rusqlite::Error>>;

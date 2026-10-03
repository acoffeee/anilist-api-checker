use rusqlite::Connection;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
pub type Db = Arc<Mutex<Connection>>;

#[derive(Serialize)]
pub struct Row {
    pub ts: i64,
    pub ok: bool,
    pub status: u16,
    pub latency_ms: i64,
}

#[derive(Deserialize)]
pub struct ListParams {
    pub limit: Option<u32>,
}
#[derive(Debug, Clone)]
pub struct PingResult {
    pub ok: bool,
    pub status: u16, // 0 = no response (timeout, DNS, connection error)
    pub latency_ms: i64,
}

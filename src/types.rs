use std::sync::{Arc, Mutex};
use rusqlite::Connection;
type Db = Arc<Mutex<Connection>>;

#[derive(Serialize)]
struct Row {
    ts: i64,
    ok: bool,
    status: u16,
    latency_ms: i64,
}

#[derive(Deserialize)]
struct ListParams {
    limit: Option<u32>,
}
struct PingResult {
    ok: bool,
    status: u16, // 0 = no response (timeout, DNS, connection error)
    latency_ms: u128,
}

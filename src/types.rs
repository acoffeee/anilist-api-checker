use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_rusqlite::rusqlite;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaasConfig {
    pub provider: String,
    pub arn: String,
    pub region: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub region: String,
}

#[derive(Debug, Deserialize)]
pub struct PingsWithLimitParams {
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ping {
    pub ok: bool,
    pub status: u16, // 0 = no response
    pub latency_ms: i64,
    pub region: String,
    pub time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct vps_config {
    pub api_key: String,
    pub region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsePing {
    pub success: bool,
    // A VPS would use this to determine when to ping again,
    // but FaaS doesn't use this.
    pub time_to_next_ping: i64,
}

#[derive( Clone)]
pub struct AppState {
    pub api_keys: Arc<Vec<String>>,
    pub db: Arc<Db>,
    pub pokers: Arc<Pokers>
}

pub type Db = tokio_rusqlite::Connection;

pub type DbResult<T> =
    Result<T, tokio_rusqlite::Error<rusqlite::Error>>;
pub struct Pokers {
    aws_lambda: bool
    //loudflare_workers: bool,
    //google_cloud_functions: bool,
    //microsoft_azura_functions: bool,
    //deno_deplou: bool,
    //oracle: bool,
    //socks5: bool,
}

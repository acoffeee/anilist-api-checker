use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use crate::database::{Db, Row};
use tokio::time::{interval, MissedTickBehavior};
const URL: &str = "https://graphql.anilist.co";

async fn poller(db: Db) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    let mut tick = interval(Duration::from_secs(60));
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        tick.tick().await;
        let r = ping(&client).await;
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

        let conn = db.lock().unwrap();
        if let Err(e) = conn.execute(
            "INSERT INTO pings (ts, ok, status, latency_ms) VALUES (?1, ?2, ?3, ?4)",
            params![ts, r.ok as i64, r.status, r.latency_ms as i64],
        ) {
            eprintln!("insert failed: {e}");
        }
    }
}
async fn ping(client: &reqwest::Client) -> PingResult {
    let body = serde_json::json!({
        "query": "query ($page: Int) { Page(page: $page, perPage: 1) { media { id } } }",
        "variables": { "page": fastrand::u32(1..=5000) }
    });
    let start = Instant::now();
    match client.post(URL).json(&body).send().await {
        Ok(r) => {
            let status = r.status();
            let text = r.text().await.unwrap_or_default();
            let latency_ms = start.elapsed().as_millis();
            let ok = status.is_success() && !text.contains("\"errors\"");
            PingResult { ok, status: status.as_u16(), latency_ms }
        }
        Err(f) => PingResult {
            ok: false,
            status: f.status().map_or(0, |s| s.as_u16()),
            latency_ms: start.elapsed().as_millis(),
        },
    }
}
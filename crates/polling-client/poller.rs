use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time::{MissedTickBehavior, interval};

use crate::database::insert_ping;
use crate::types::{Db, PingResult};

const URL: &str = "https://graphql.anilist.co";

pub async fn poller(db: Db) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    let mut tick = interval(Duration::from_secs(60));
    tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        tick.tick().await;
        let r = ping(&client).await;
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let conn = db.lock().unwrap();
        insert_ping(&conn, ts, r.ok, r.status, r.latency_ms).unwrap_or_else(|e| {
            eprintln!("Failed to insert ping result: {}", e);
        });
    }
}
pub async fn ping(client: &reqwest::Client) -> PingResult {
    let body = serde_json::json!({
        "query": "query ($page: Int) { Page(page: $page, perPage: 1) { media { id } } }",
        "variables": { "page": fastrand::u32(1..=5000) }
    });
    let start = Instant::now();
    match client.post(URL).json(&body).send().await {
        Ok(r) => {
            let status = r.status();
            let text = r.text().await.unwrap_or_default();
            let latency_ms = i64::try_from(start.elapsed().as_millis()).unwrap_or(i64::MAX);
            let ok = status.is_success() && !text.contains("\"errors\"");
            PingResult {
                ok,
                status: status.as_u16(),
                latency_ms,
            }
        }
        Err(f) => PingResult {
            ok: false,
            status: f.status().map_or(0, |s| s.as_u16()),
            latency_ms: i64::try_from(start.elapsed().as_millis()).unwrap_or(i64::MAX),
        },
    }
}

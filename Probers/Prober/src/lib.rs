// "Pokes" the anilist api to check if its up
//dont poke the bear too much!
use reqwest;
use std::time::Instant;
use serde::{Deserialize, Serialize};
use std::time::Duration;
pub const URL: &str = "https://graphql.anilist.co";


#[derive(Serialize, Deserialize)]
pub struct PingResult {
    pub time: u64,
    pub ok: bool,
    pub status: u16,
    pub latency_ms: i64,
}

pub async fn prober(client: &reqwest::Client, ) -> PingResult {
    let body = serde_json::json!({
        "query": "query ($page: Int) { Page(page: $page, perPage: 1) { media { id } } }",
        "variables": { "page": fastrand::u32(1..=5000) }
    });
    let start = Instant::now();
    match client.post(URL).json(&body).send().await {
        Ok(r) => {
            let end = Instant::now();
            let time = end.duration_since(start).as_secs();
            let status = r.status();
            let text = r.text().await.unwrap_or_default();
            let latency_ms = i64::try_from(start.elapsed().as_millis()).unwrap_or(i64::MAX);
            let ok = status.is_success() && !text.contains("\"errors\"");
            PingResult {
                time: time,
                ok,
                status: status.as_u16(),
                latency_ms
            }
        }
        Err(f) =>  {
            let end = Instant::now();
            let time = end.duration_since(start).as_secs();
            PingResult {
                time: time,
                ok: false,
                status: f.status().map_or(0, |s| s.as_u16()),
                latency_ms: i64::try_from(start.elapsed().as_millis()).unwrap_or(i64::MAX),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

// "Pokes" the anilist api to check if its up
//dont poke the bear too much!
use reqwest;
use std::time::Instant;
use serde::{Deserialize, Serialize};

pub const URL: &str = "https://graphql.anilist.co";


#[derive(Serialize, Deserialize)]
pub struct PingResult {
    pub ok: bool,
    pub status: u16,
    pub latency_ms: i64,
}
pub async fn poker(client: &reqwest::Client) -> PingResult {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

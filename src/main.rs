use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
    response::Html
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tokio::time::{interval, MissedTickBehavior};

type Db = Arc<Mutex<Connection>>;

// ---- PingResult and ping() exactly as in the previous message ----

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

const URL: &str = "https://graphql.anilist.co";

struct PingResult {
    ok: bool,
    status: u16, // 0 = no response (timeout, DNS, connection error)
    latency_ms: u128,
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
fn row_from(r: &rusqlite::Row) -> rusqlite::Result<Row> {
    Ok(Row {
        ts: r.get(0)?,
        ok: r.get::<_, i64>(1)? != 0,
        status: r.get(2)?,
        latency_ms: r.get(3)?,
    })
}

// GET /latency?limit=60
async fn list(
    State(db): State<Db>,
    Query(p): Query<ListParams>,
) -> Result<Json<Vec<Row>>, StatusCode> {
    let limit = p.limit.unwrap_or(60).min(43200);
    let conn = db.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT ts, ok, status, latency_ms FROM pings ORDER BY ts DESC LIMIT ?1")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let rows = stmt
        .query_map(params![limit], row_from)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(rows))
}

// GET /latency/latest
async fn latest(State(db): State<Db>) -> Result<Json<Row>, StatusCode> {
    let conn = db.lock().unwrap();
    conn.query_row(
        "SELECT ts, ok, status, latency_ms FROM pings ORDER BY ts DESC LIMIT 1",
        [],
        row_from,
    )
    .map(Json)
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })
}

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

#[tokio::main]
async fn main() {
    //create da database
    let conn = Connection::open("pings.db").unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS pings (
            ts INTEGER NOT NULL,
            ok INTEGER NOT NULL,
            status INTEGER NOT NULL,
            latency_ms INTEGER NOT NULL
        )",
        [],
    )
    .unwrap();
    let db: Db = Arc::new(Mutex::new(conn));

    tokio::spawn(poller(db.clone()));

    let app = Router::new()
        .route("/", get(|| async { Html(include_str!("../index.html")) }))
        .route("/latency", get(list))
        .route("/latency/latest", get(latest))
        .with_state(db);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
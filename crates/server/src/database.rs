use crate::types::{Db, Row};
use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};
pub type Db = Arc<Mutex<Connection>>;
pub fn start_db() -> Db {
    let conn =
        Connection::open(&std::env::var("DATABASE_PATH").unwrap_or_else(|_| "pings.db".into()))
            .unwrap();
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
    db
}
pub fn row_from(r: &rusqlite::Row) -> rusqlite::Result<Row> {
    Ok(Row {
        ts: r.get(0)?,
        ok: r.get::<_, i64>(1)? != 0,
        status: r.get(2)?,
        latency_ms: r.get(3)?,
    })
}
pub fn fetch_latest_rows(conn: &Connection, limit: u32) -> rusqlite::Result<Vec<Row>> {
    let mut stmt =
        conn.prepare("SELECT ts, ok, status, latency_ms FROM pings ORDER BY ts DESC LIMIT ?1")?;
    let rows = stmt
        .query_map(params![limit], row_from)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}
pub fn insert_ping(
    conn: &Connection,
    ts: i64,
    ok: bool,
    status: u16,
    latency_ms: i64,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO pings (ts, ok, status, latency_ms) VALUES (?1, ?2, ?3, ?4)",
        params![ts, ok as i64, status, latency_ms],
    )?;
    Ok(())
}
pub fn fetch_latest_row(conn: &Connection) -> rusqlite::Result<Row> {
    conn.query_row(
        "SELECT ts, ok, status, latency_ms FROM pings ORDER BY ts DESC LIMIT 1",
        [],
        row_from,
    )
}

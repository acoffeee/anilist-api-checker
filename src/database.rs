use rusqlite::{params, Connection};
fn start_db() -> Db {
let conn = Connection::open(&std::env::var("DATABASE_PATH").unwrap_or_else(|_| "pings.db".into())).unwrap();
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
fn row_from(r: &rusqlite::Row) -> rusqlite::Result<Row> {
    Ok(Row {
        ts: r.get(0)?,
        ok: r.get::<_, i64>(1)? != 0,
        status: r.get(2)?,
        latency_ms: r.get(3)?,
    })
}
fn fetch_latest_rows(conn: &Connection, limit: u32) -> rusqlite::Result<Vec<Row>> {
    let mut stmt = conn.prepare(
        "SELECT ts, ok, status, latency_ms FROM pings ORDER BY ts DESC LIMIT ?1",
    )?;
    let rows = stmt
        .query_map(params![limit], row_from)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}
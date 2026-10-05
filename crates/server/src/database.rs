use crate::types::{Db, Row};
use tokio_rusqlite::rusqlite;

// ---------- public API: the only thing the api code ever touches ----------

/// Error returned by every query: tokio_rusqlite's wrapper around rusqlite::Error.
pub type DbResult<T> = Result<T, tokio_rusqlite::Error<rusqlite::Error>>;

pub async fn start_db() -> Db {
    let path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "pings.db".into());
    let db = Db::open(path).await.expect("failed to open database");
    init_schema(&db).await;
    db
}

async fn init_schema(db: &Db) {
    db.call(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pings (
                ts INTEGER NOT NULL,
                ok INTEGER NOT NULL,
                status INTEGER NOT NULL,
                latency_ms INTEGER NOT NULL,
                region TEXT NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_region_ts ON pings (region, ts)",
            [],
        )?;
        Ok::<_, rusqlite::Error>(())
    })
    .await
    .expect("failed to initialise schema");
}

/// Most recent ping, or None if the table is empty.
pub async fn fetch_latest_row(db: &Db) -> DbResult<Option<Row>> {
    db.call(|conn| sql::fetch_latest_row(conn)).await
}

/// The `limit` most recent pings, newest first.
pub async fn fetch_latest_rows(db: &Db, limit: u32) -> DbResult<Vec<Row>> {
    db.call(move |conn| sql::fetch_latest_rows(conn, limit)).await
}

pub async fn insert_ping(
    db: &Db,
    ts: i64,
    ok: bool,
    status: u16,
    latency_ms: i64,
    region: String,
) -> DbResult<()> {
    db.call(move |conn| sql::insert_ping(conn, ts, ok, status, latency_ms, &region))
        .await
}

// ---------- private: raw rusqlite, same names, runs on the connection thread ----------

mod sql {
    use crate::types::Row;
    use tokio_rusqlite::rusqlite::{self, params, Connection, OptionalExtension};

    pub(super) fn row_from(r: &rusqlite::Row) -> rusqlite::Result<Row> {
        Ok(Row {
            ts: r.get(0)?,
            ok: r.get::<_, i64>(1)? != 0,
            status: r.get(2)?,
            latency_ms: r.get(3)?,
        })
    }

    pub(super) fn fetch_latest_rows(conn: &Connection, limit: u32) -> rusqlite::Result<Vec<Row>> {
        let mut stmt = conn
            .prepare("SELECT ts, ok, status, latency_ms FROM pings ORDER BY ts DESC LIMIT ?1")?;
        let rows = stmt
            .query_map(params![limit], row_from)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub(super) fn fetch_latest_row(conn: &Connection) -> rusqlite::Result<Option<Row>> {
        conn.query_row(
            "SELECT ts, ok, status, latency_ms FROM pings ORDER BY ts DESC LIMIT 1",
            [],
            row_from,
        )
        .optional()
    }

    pub(super) fn insert_ping(
        conn: &Connection,
        ts: i64,
        ok: bool,
        status: u16,
        latency_ms: i64,
        region: &str,
    ) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT INTO pings (ts, ok, status, latency_ms, region) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![ts, ok as i64, status, latency_ms, region],
        )?;
        Ok(())
    }
}

// ---------- tests ----------

/// Fresh in-memory database with the schema applied. Shared with handler tests.
#[cfg(test)]
pub(crate) async fn test_db() -> Db {
    let db = Db::open_in_memory().await.unwrap();
    init_schema(&db).await;
    db
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn seed(db: &Db, ts: i64) {
        insert_ping(db, ts, true, 200, 42, "local".into())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn latest_row_is_none_when_table_is_empty() {
        let db = test_db().await;
        assert!(fetch_latest_row(&db).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn latest_rows_is_empty_when_table_is_empty() {
        let db = test_db().await;
        assert!(fetch_latest_rows(&db, 10).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn insert_then_fetch_roundtrips_every_field() {
        let db = test_db().await;
        insert_ping(&db, 1000, false, 503, 250, "eu-west-1".into())
            .await
            .unwrap();

        let row = fetch_latest_row(&db).await.unwrap().unwrap();
        assert_eq!(row.ts, 1000);
        assert!(!row.ok);
        assert_eq!(row.status, 503);
        assert_eq!(row.latency_ms, 250);
    }

    #[tokio::test]
    async fn latest_row_is_the_newest_by_ts_not_insertion_order() {
        let db = test_db().await;
        for ts in [10, 30, 20] {
            seed(&db, ts).await;
        }
        assert_eq!(fetch_latest_row(&db).await.unwrap().unwrap().ts, 30);
    }

    #[tokio::test]
    async fn latest_rows_are_newest_first_and_respect_limit() {
        let db = test_db().await;
        for ts in 1..=5 {
            seed(&db, ts).await;
        }
        let ts: Vec<i64> = fetch_latest_rows(&db, 3)
            .await
            .unwrap()
            .iter()
            .map(|r| r.ts)
            .collect();
        assert_eq!(ts, vec![5, 4, 3]);
    }

    #[tokio::test]
    async fn limit_larger_than_row_count_returns_everything() {
        let db = test_db().await;
        for ts in 1..=3 {
            seed(&db, ts).await;
        }
        assert_eq!(fetch_latest_rows(&db, 1000).await.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn schema_init_is_idempotent() {
        let db = test_db().await;
        seed(&db, 1).await;
        init_schema(&db).await; // second run must not fail or wipe data
        assert_eq!(fetch_latest_rows(&db, 10).await.unwrap().len(), 1);
    }
}
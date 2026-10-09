use crate::types::*;
use tokio_rusqlite::rusqlite;
// ---------- public API ----------

pub async fn start_db() -> Db {
    let path = std::env::var("DATABASE_PATH")
        .unwrap_or_else(|_| "pings.db".into());

    let db = Db::open(path)
        .await
        .expect("failed to open database");

    init_schema(&db).await;

    db
}

async fn init_schema(db: &Db) {
    db.call(|conn| {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS region (
                region TEXT PRIMARY KEY
            )",
            []
        );
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pings (
                time INTEGER NOT NULL,
                ok INTEGER NOT NULL,
                status INTEGER NOT NULL,
                latency_ms INTEGER NOT NULL,
                region TEXT REFERENCES region(region)
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pings_region
             ON pings (region)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS faas_config (
                provider TEXT PRIMARY KEY,
                arn TEXT NOT NULL,
                region TEXT REFERENCES region(region)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS vps_config (
                api_key TEXT PRIMARY KEY,
                region TEXT REFERENCES region(region)
            )",
            [],
        )?;

        Ok::<_, rusqlite::Error>(())
    })
    .await
    .expect("failed to initialise schema");
}

/// Most recent ping, or None if the table is empty.
pub async fn fetch_latest_ping(db: &Db) -> DbResult<Option<Ping>> {
    db.call(|conn| sql::fetch_latest_ping(conn)).await
}

/// The `limit` most recent pings, newest first.
pub async fn fetch_latest_pings(
    db: &Db,
    limit: u32,
) -> DbResult<Vec<Ping>> {
    db.call(move |conn| {
        sql::fetch_latest_pings(conn, limit)
    })
    .await
}

pub async fn insert_ping(
    db: &Db,
    ping: Ping
) -> DbResult<()> {
    db.call(move |conn| {
        sql::insert_ping(
            conn,
            ping.time,
            ping.ok,
            ping.status,
            ping.latency_ms,
            ping.region

        )
    })
    .await
}

pub async fn fetch_faas_provider_configs(
    db: &Db,
    provider: String,
) -> Result<Vec<FaasConfig>, tokio_rusqlite::Error> {
    db.call(move |conn| {
        sql::fetch_faas_provider_configs(conn, &provider)
    })
    .await
}

pub async fn insert_faas_config(
    db: &Db,
    provider: String,
    arn: String,
    region: String
) -> DbResult<()> {
    db.call(move |conn| {
        sql::insert_faas_config(
            conn,
            &provider,
            &arn,
            region
        )
    })
    .await
}

// ---------- private: raw rusqlite ----------

mod sql {
    use crate::types::{FaasConfig, Ping};
    use tokio_rusqlite::rusqlite::{
        self,
        params,
        Connection,
        OptionalExtension,
    };

    pub(super) fn ping_from(
        row: &rusqlite::Row<'_>,
    ) -> rusqlite::Result<Ping> {
        Ok(Ping {
            time: row.get(0)?,
            ok: row.get::<_, i64>(1)? != 0,
            status: row.get(2)?,
            latency_ms: row.get(3)?,
            region: row.get(4)?,
        })
    }

    pub(super) fn fetch_latest_pings(
        conn: &Connection,
        limit: u32,
    ) -> rusqlite::Result<Vec<Ping>> {
        let mut stmt = conn.prepare(
            "SELECT
                time,
                ok,
                status,
                latency_ms,
                region,
                region_id
             FROM pings
             ORDER BY time DESC
             LIMIT ?1",
        )?;

        let rows = stmt
            .query_map(params![limit], ping_from)?
            .collect::<rusqlite::Result<Vec<Ping>>>()?;

        Ok(rows)
    }

    pub(super) fn fetch_latest_ping(
        conn: &Connection,
    ) -> rusqlite::Result<Option<Ping>> {
        conn.query_row(
            "SELECT
                time,
                ok,
                status,
                latency_ms,
                region,
                region_id
             FROM pings
             ORDER BY time DESC
             LIMIT 1",
            [],
            ping_from,
        )
        .optional()
    }

    pub(super) fn insert_ping(
        conn: &Connection,
        time: i64,
        ok: bool,
        status: i64,
        latency_ms: i64,
        region: String,
    ) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT INTO pings (
                time,
                ok,
                status,
                latency_ms,
                region,
            )
            VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                time,
                ok as i64,
                status,
                latency_ms,
                region
            ],
        )?;

        Ok(())
    }

    pub(super) fn fetch_faas_provider_configs(
        conn: &Connection,
        provider: &str,
    ) -> rusqlite::Result<Vec<FaasConfig>> {
        let mut stmt = conn.prepare(
            "SELECT provider, arn, region_id
             FROM faas_config
             WHERE provider = ?1"
            );
            let rows = stmt?
            .query_map(params![provider], |row| {
                Ok( FaasConfig {
                    provider: row.get(0)?,
                    arn: row.get(1)?,
                    region: row.get(2)?
                })
            })?
            .collect::<rusqlite::Result<Vec<FaasConfig>>>()?;
        Ok(rows)
    }

    pub(super) fn insert_faas_config(
        conn: &Connection,
        provider: &str,
        arn: &str,
        region: String
    ) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT INTO faas_config (
                provider,
                arn,
                region
            )
            VALUES (?1, ?2, ?3)",
            params![provider, arn, region],
        )?;

        Ok(())
    }
}
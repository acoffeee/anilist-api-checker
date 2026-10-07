use axum::{Json, http::StatusCode, extract::State};
use crate::database::insert_ping as db_insert_ping;
use crate::types::{Ping, ResponsePing};
use crate::types::Db;
//i was gonna do a whole thing about moving the db insert onto its own thread so that this function could return immediately, but like its probably fine and if its not i can look into it
pub async fn insert_ping(
    State(db): State<Db>,
    Json(payload): Json<Ping>
) -> Json<ResponsePing> {
    db_insert_ping(&db, payload.time, payload.ok, payload.status, payload.latency_ms, payload.region)
        .await;
    Json(ResponsePing {
        success: true,
        time_to_next_ping: 0, // Replace with actual value if available
    })
}

use axum::{Json, http::StatusCode};
use database::insert_ping as db_insert_ping;
use chrono::Utc;
use types::{Ping, responsePing};
//i was gonna do a whole thing about moving the db insert onto its own thread so that this function could return immediately, but like its probably fine and if its not i can look into it
pub async fn insert_ping(
    Json(payload): Json<Ping>
) -> responsePing {
    match db_insert_ping(payload.time, payload.ok, payload.status, payload.latency_ms, payload.region)
        .await
        .unwrap()
    {
        Ok(_) => responsePing {
            success: true,
            time_to_next_ping: 0, // Replace with actual value if available
        },
        Err(_) => responsePing {
            success: false,
            time_to_next_ping: 0, // Replace with actual value if available
        },
    }
}

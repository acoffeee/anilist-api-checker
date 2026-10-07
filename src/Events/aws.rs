use aws_sdk_lambda::Client;
use aws_config::Region;
use std::sync::Arc;
use crate::database::{fetch_faas_provider_configs, insert_ping};
use serde_json::Value;
use crate::types::{Db, DbResult, Pokers};
pub async fn run_job(db: &Db) {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    let client_arc = Arc::new(client);
    let aws_configs = match fetch_faas_provider_configs(&db, String::from("aws")).await {
        Ok(config) => config,
        Err(tokio_rusqlite::Error::Error(e)) => {
            eprintln!("{:?}", e);
            return;
        }
    };
    for config in aws_configs{
        tokio::spawn ( async move {
            let result = client
                .invoke()
                .function_name(&config.arn)
                .send()
                .await;

            let output = match result {
                Ok(output) => output,
                Err(e) => {
                    eprintln!(
                        "Failed to invoke {}: {:?}",
                        config.arn,
                        e
                    );
                    return;
                }
            };

            let payload = match output.payload {
                Some(payload) => payload,
                None => {
                    eprintln!(
                        "Lambda {} returned no payload",
                        config.arn
                    );
                    return;
                }
            };

            let response: Value = match serde_json::from_slice(&payload) {
                Ok(value) => value,
                Err(e) => {
                    eprintln!(
                        "Invalid Lambda response from {}: {:?}",
                        config.arn,
                        e
                    );
                    return;
                }
            };

            let time = match response["time"].as_i64() {
                Some(value) => value,
                None => {
                    eprintln!("Missing/invalid `time`");
                    return;
                }
            };

            let ok = match response["ok"].as_bool() {
                Some(value) => value,
                None => {
                    eprintln!("Missing/invalid `ok`");
                    return;
                }
            };

            let status = match response["status"].as_u64() {
                Some(value) => value as u16,
                None => {
                    eprintln!("Missing/invalid `status`");
                    return;
                }
            };

            let latency_ms = match response["latency_ms"].as_i64() {
                Some(value) => value,
                None => {
                    eprintln!("Missing/invalid `latency_ms`");
                    return;
                }
            };

            if let Err(e) = insert_ping(
                &db,
                time,
                ok,
                status,
                latency_ms,
                config.region,
            )
            .await
            {
                eprintln!("Failed to insert ping: {:?}", e);
            }
        });
    }
}
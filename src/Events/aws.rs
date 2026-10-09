use aws_sdk_lambda::Client;
use aws_config::Region;
use std::sync::Arc;
use crate::database::{fetch_faas_provider_configs, insert_ping};
use serde_json::Value;
use crate::types::{Db, DbResult, Pokers, Ping};
pub async fn run_job(db: Db) {
    let config = aws_config::load_from_env().await;

    let client = Client::new(&config);
    let aws_configs = match fetch_faas_provider_configs(&db, String::from("aws")).await {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{:?}", e);
            return;
        }
    };
    for config in aws_configs{
        let db = db.clone();
        //ngl we kinda hoping its cheap to clone
        let client = client.clone();
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

            let response: serde_json::Value = match output.payload {
                //wont throw an error trust
                Some(payload) => serde_json::from_slice(&payload.into_inner()).unwrap(),
                None => {
                    eprintln!(
                        "Lambda {} returned no payload",
                        config.arn
                    );
                    return;
                }
            };
            let ping = Ping {
                time:  response["time"].as_i64().unwrap(),
                ok:  response["ok"].as_bool().unwrap(),
                status:  response["status"].as_i64().unwrap(),
                latency_ms:  response["latency_ms"].as_i64().unwrap(),
                region: config.region
            };
            
            if let Err(e) = insert_ping(&db,ping).await
            {
                eprintln!("Failed to insert ping: {:?}", e);
            };
        });
    }
}
use lambda_runtime::{Error, LambdaEvent};
use serde_json::Value;
use Poker::{PingResult, poker};
use reqwest::Client;
/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/

pub(crate)async fn function_handler(event: LambdaEvent<Value>) -> Result<(), Error> {
    // Extract some useful information from the request
    let callback_url = event.payload.get("callback_url").and_then(|v| v.as_str()).unwrap_or_default();
    let api_key = event.payload.get("api_key").and_then(|v| v.as_str()).unwrap_or_default();
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;
    let ping_result: PingResult = poker(&client).await;
    let _ = client.post(callback_url)
        .header("x-api-key", api_key)
        .json(&ping_result)
        .send()
        .await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_runtime::{Context, LambdaEvent};

    #[tokio::test]
    async fn test_event_handler() {
        let event = LambdaEvent::new(Value::default(), Context::default());
        let response = function_handler(event).await.unwrap();
        assert_eq!((), response);
    }
}

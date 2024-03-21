use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};

async fn run_lambda(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (event, _context) = event.into_parts();

    let base_url = event["url"].as_str().unwrap_or_default();
    let result = fetch_data::get_weekly_stats(base_url).await;
    Ok(json!(result))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(service_fn(run_lambda)).await
}

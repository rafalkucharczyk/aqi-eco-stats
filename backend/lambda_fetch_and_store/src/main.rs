use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};

#[cfg(not(test))]
use fetch_data::core::get_weekly_stats;
#[cfg(test)]
use fetch_data::mock_core::get_weekly_stats;

#[cfg(not(test))]
async fn store_weekly_stats(
    url: &str,
    weekly_stats: &json_structs::output::WeeklyStats,
) -> Result<(), Error> {
    let db_client = store_data::core::DynamoDBClient::connect().await;
    store_data::core::store(url, weekly_stats, db_client).await?;

    Ok(())
}

async fn run_lambda(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (event, _context) = event.into_parts();

    let base_url_json = event.get("url").ok_or("no url")?;
    let base_url = base_url_json.as_str().unwrap();

    let weekly_stats = get_weekly_stats(base_url).await;

    #[cfg(not(test))]
    store_weekly_stats(base_url, &weekly_stats).await?;

    Ok(json!(weekly_stats))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(service_fn(run_lambda)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use json_structs::output::WeeklyStats;
    use mockall::predicate;
    use serial_test::serial;

    #[serial]
    #[tokio::test]
    async fn run_lambda_when_url_is_defined_then_returns_get_weekly_stats_result() {
        let ctx = fetch_data::mock_core::get_weekly_stats_context();
        ctx.expect()
            .with(predicate::eq("foo"))
            .times(1)
            .returning(|_url| WeeklyStats {
                start: 5,
                ..WeeklyStats::default()
            });

        let r = run_lambda(LambdaEvent::new(
            json!({"url": "foo"}),
            lambda_runtime::Context::default(),
        ))
        .await;

        assert!(r.is_ok());
        assert_eq!(
            r.unwrap(),
            json!(WeeklyStats {
                start: 5,
                ..WeeklyStats::default()
            })
        )
    }

    #[serial]
    #[tokio::test]
    async fn run_lambda_when_url_is_missing_then_returns_error() {
        let ctx = fetch_data::mock_core::get_weekly_stats_context();
        ctx.expect().times(0);

        let r = run_lambda(LambdaEvent::new(
            json!({}),
            lambda_runtime::Context::default(),
        ))
        .await;

        assert!(r.is_err());
        assert_eq!(r.err().unwrap().to_string(), "no url");
    }
}

pub mod core {
    use anyhow::Error;
    use async_trait::async_trait;
    use aws_config::{BehaviorVersion, SdkConfig};
    use aws_sdk_dynamodb::types::AttributeValue;
    use json_structs::output::WeeklyStats;
    #[cfg(test)]
    use mockall::automock;
    use serde_dynamo::to_item;
    use std::{
        collections::HashMap,
        env,
        time::{SystemTime, UNIX_EPOCH},
    };

    pub type DbItem = Option<HashMap<String, AttributeValue>>;

    #[cfg_attr(test, automock)]
    #[async_trait]
    pub trait StoreWeeklyStats {
        async fn put_to_db(&self, url: &str, item: DbItem) -> anyhow::Result<()>;
    }

    pub async fn store(
        url: &str,
        weekly_stats: &WeeklyStats,
        store: impl StoreWeeklyStats,
    ) -> anyhow::Result<()> {
        let item = to_item(weekly_stats)?;

        store.put_to_db(url, Some(item)).await
    }

    pub struct DynamoDBClient {
        client: aws_sdk_dynamodb::Client,
    }

    impl DynamoDBClient {
        async fn get_sdk_config(run_local: bool) -> SdkConfig {
            let mut sdk_config_loader = aws_config::defaults(BehaviorVersion::latest());
            if run_local {
                sdk_config_loader = sdk_config_loader.test_credentials();
            }
            sdk_config_loader.load().await
        }

        async fn get_dynamodb_config(
            run_local: bool,
            sdk_config: &SdkConfig,
        ) -> aws_sdk_dynamodb::Config {
            let mut dynamodb_config_builder = aws_sdk_dynamodb::config::Builder::from(sdk_config);
            if run_local {
                dynamodb_config_builder =
                    dynamodb_config_builder.endpoint_url("http://localhost:8000")
            }
            dynamodb_config_builder.build()
        }

        pub async fn connect() -> Self {
            let run_local = env::var("RUN_LOCAL").is_ok();

            let sdk_config = Self::get_sdk_config(run_local).await;

            DynamoDBClient {
                client: aws_sdk_dynamodb::Client::from_conf(
                    Self::get_dynamodb_config(run_local, &sdk_config).await,
                ),
            }
        }
    }

    #[async_trait]
    impl StoreWeeklyStats for DynamoDBClient {
        async fn put_to_db(&self, url: &str, item: DbItem) -> anyhow::Result<()> {
            let env_type = env::var("ENV_TYPE").unwrap_or("dev".to_string());
            let request = self
                .client
                .put_item()
                .table_name(format!("stats-{}", env_type))
                .item("data", AttributeValue::M(item.unwrap()))
                .item("url", AttributeValue::S(url.to_string()))
                .item(
                    "time",
                    AttributeValue::N(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                            .to_string(),
                    ),
                );
            request
                .send()
                .await
                .map_err(|x| Error::msg(format!("DynamoDB failed: {:?}", x.into_source())))?;

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use aws_sdk_dynamodb::types::AttributeValue;
    use json_structs::output::{LocationMinMax, MeasurementMinMax, WeeklyStats};
    use mockall::predicate;

    use super::core::*;

    #[tokio::test]
    async fn store_when_empty_weekly_stats_given_then_puts_to_db_zeroed_values() {
        let mut mock = MockStoreWeeklyStats::new();
        let test_url = "some_url";
        let db_item = HashMap::from([
            ("locations".into(), AttributeValue::L([].to_vec())),
            ("start".into(), AttributeValue::N("0".into())),
            ("end".into(), AttributeValue::N("0".into())),
        ]);

        mock.expect_put_to_db()
            .with(predicate::eq(test_url), predicate::eq(Some(db_item)))
            .returning(|_url, _item| Ok(()));

        let result = store(test_url, &WeeklyStats::default(), mock).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn store_when_weekly_stats_given_then_puts_to_db_corresponding_values() {
        let mut mock_store = MockStoreWeeklyStats::new();
        let test_url = "some_url";
        let db_item = HashMap::from([
            (
                "locations".into(),
                AttributeValue::L(
                    [AttributeValue::M(HashMap::from([
                        ("name".into(), AttributeValue::S("foo".into())),
                        ("start_date".into(), AttributeValue::N(1.to_string())),
                        ("end_date".into(), AttributeValue::N(2.to_string())),
                        (
                            "pm10".into(),
                            AttributeValue::M(HashMap::from([
                                ("min".into(), AttributeValue::N(1.0.to_string())),
                                ("max".into(), AttributeValue::N(5.0.to_string())),
                            ])),
                        ),
                        (
                            "pm25".into(),
                            AttributeValue::M(HashMap::from([
                                ("min".into(), AttributeValue::N(3.0.to_string())),
                                ("max".into(), AttributeValue::N(8.0.to_string())),
                            ])),
                        ),
                    ]))]
                    .to_vec(),
                ),
            ),
            ("start".into(), AttributeValue::N(111.to_string())),
            ("end".into(), AttributeValue::N(222.to_string())),
        ]);

        mock_store
            .expect_put_to_db()
            .with(predicate::eq(test_url), predicate::eq(Some(db_item)))
            .returning(|_url, _item| Ok(()));

        let weekly_stats = WeeklyStats {
            start: 111,
            end: 222,
            locations: Vec::from([LocationMinMax {
                name: "foo".into(),
                start_date: 1,
                end_date: 2,
                pm10: MeasurementMinMax { min: 1.0, max: 5.0 },
                pm25: MeasurementMinMax { min: 3.0, max: 8.0 },
            }]),
        };

        let result = store(test_url, &weekly_stats, mock_store).await;
        assert!(result.is_ok());
    }
}

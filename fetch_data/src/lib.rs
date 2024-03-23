pub mod json_structs;

use mockall::automock;

mod utils {
    use crate::json_structs::deserializable::{parse_json, Location};
    use std::cmp::Ordering;

    #[derive(Clone)]
    pub struct RawSensorData {
        pub name: String,
        pub json: String,
    }

    async fn get_url_text(url: &str) -> Result<String, reqwest::Error> {
        println!("{}", url);
        let body = reqwest::get(url).await?.text().await?;

        Ok(body)
    }

    async fn get_locations(base_url: &str) -> Vec<Location> {
        let data = get_url_text(format!("{}/map/data.json", base_url).as_str())
            .await
            .unwrap_or_default();
        parse_json::<Vec<Location>>(&data)
    }

    pub async fn get_sensor_raw_data(base_url: &str) -> Vec<RawSensorData> {
        let tasks = get_locations(base_url).await.into_iter().map(|x| {
            let url = String::from(base_url);
            tokio::spawn(async move {
                RawSensorData {
                    json: get_url_text(
                        format!(
                            "{}{}/graph_data.json?type=pm&range=week&ma_h=24",
                            url, x.path
                        )
                        .as_str(),
                    )
                    .await
                    .unwrap(),
                    name: x.description,
                }
            })
        });

        futures::future::join_all(tasks)
            .await
            .iter()
            .map(|x| x.as_ref().unwrap().clone())
            .collect()
    }

    fn map_measurements(
        series: &std::collections::HashMap<u32, Option<f32>>,
    ) -> impl Iterator<Item = f32> + '_ {
        series
            .iter()
            .filter(|x| x.1.is_some())
            .map(|x| x.1.unwrap_or_default())
    }

    pub fn get_min(series: &std::collections::HashMap<u32, Option<f32>>) -> f32 {
        map_measurements(series)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .unwrap_or_default()
    }

    pub fn get_max(series: &std::collections::HashMap<u32, Option<f32>>) -> f32 {
        map_measurements(series)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .unwrap_or_default()
    }
}

#[automock]
pub mod core {
    use crate::json_structs::deserializable::{parse_json, SensorData};
    use crate::json_structs::serializable::{LocationMinMax, MeasurementMinMax, WeeklyStats};
    use crate::utils::{get_max, get_min, get_sensor_raw_data};

    pub async fn get_weekly_stats(base_url: &str) -> WeeklyStats {
        let sensors_raw_data = get_sensor_raw_data(base_url).await;

        let locations: Vec<LocationMinMax> = sensors_raw_data
            .iter()
            .filter_map(|x| {
                let sensor_data = &parse_json::<SensorData>(&x.json);
                let data = &sensor_data.data;

                if sensor_data != &SensorData::default() {
                    Some(LocationMinMax {
                        name: String::from(&x.name),
                        start_date: sensor_data.start,
                        end_date: sensor_data.end,
                        pm25: MeasurementMinMax {
                            min: get_min(&data.pm25),
                            max: get_max(&data.pm25),
                        },
                        pm10: MeasurementMinMax {
                            min: get_min(&data.pm10),
                            max: get_max(&data.pm10),
                        },
                    })
                } else {
                    None
                }
            })
            .collect();

        WeeklyStats {
            start: locations
                .iter()
                .min_by_key(|x| x.start_date)
                .unwrap_or(&LocationMinMax::default())
                .start_date,
            end: locations
                .iter()
                .max_by_key(|x| x.end_date)
                .unwrap_or(&LocationMinMax::default())
                .end_date,
            locations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::core::*;
    use crate::json_structs::serializable::WeeklyStats;
    use serde_json::json;

    #[tokio::test]
    async fn get_weekly_stats_when_valid_data_is_fetched_then_returns_result() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        let mock_data_json = server
            .mock("GET", "/map/data.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!([{"description": "foobar", "path": "/foo/bar"}]).to_string())
            .expect(1)
            .create_async()
            .await;

        let mock_graph_data_json = server
            .mock("GET", "/foo/bar/graph_data.json?type=pm&range=week&ma_h=24")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({"start": 1, "end": 2, "data":{"pm10":{"1": 10.5, "2": 14.5}, "pm25":{"1": 13.2, "2": 3.4}}}).to_string())
            .expect(1)
            .create_async().await;

        let result = get_weekly_stats(&url).await;

        mock_data_json.assert_async().await;
        mock_graph_data_json.assert_async().await;

        assert_eq!(result.start, 1);
        assert_eq!(result.end, 2);
        assert_eq!(result.locations.len(), 1);

        let first_location = result.locations.first().unwrap();
        assert_eq!(first_location.name, String::from("foobar"));
        assert_eq!(first_location.pm10.min, 10.5);
        assert_eq!(first_location.pm10.max, 14.5);
        assert_eq!(first_location.pm25.min, 3.4);
        assert_eq!(first_location.pm25.max, 13.2);
    }

    #[tokio::test]
    async fn get_weekly_stats_when_invalid_map_data_is_fetched_then_returns_empty_result() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        let mock_data_json = server
            .mock("GET", "/map/data.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!([{"description": "foobar"}]).to_string())
            .expect(1)
            .create_async()
            .await;

        let mock_graph_data_json = server
            .mock("GET", mockito::Matcher::Any)
            .expect(0)
            .create_async()
            .await;

        let result = get_weekly_stats(&url).await;

        mock_data_json.assert_async().await;
        mock_graph_data_json.assert_async().await;

        assert_eq!(result, WeeklyStats::default());
    }

    #[tokio::test]
    async fn get_weekly_stats_when_invalid_graph_data_is_fetched_then_returns_empty_result() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        let mock_data_json = server
            .mock("GET", "/map/data.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!([{"description": "foobar", "path": "/foo/bar"}]).to_string())
            .expect(1)
            .create_async()
            .await;

        let mock_graph_data_json = server
            .mock("GET", "/foo/bar/graph_data.json?type=pm&range=week&ma_h=24")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({"start": 1}).to_string())
            .expect(1)
            .create_async()
            .await;

        let result = get_weekly_stats(&url).await;

        mock_data_json.assert_async().await;
        mock_graph_data_json.assert_async().await;

        assert_eq!(result, WeeklyStats::default());
    }
}

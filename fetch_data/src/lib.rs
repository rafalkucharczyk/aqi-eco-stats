mod json_structs;
use crate::json_structs::deserializable::{parse_json, Location, SensorData};
use crate::json_structs::serializable::{LocationMinMax, MeasurementMinMax, WeeklyStats};

use std::cmp::Ordering;

#[derive(Clone)]
struct RawSensorData {
    name: String,
    json: String,
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

async fn get_sensor_raw_data(base_url: &str) -> Vec<RawSensorData> {
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

fn get_min(series: &std::collections::HashMap<u32, Option<f32>>) -> f32 {
    map_measurements(series)
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
        .unwrap_or_default()
}

fn get_max(series: &std::collections::HashMap<u32, Option<f32>>) -> f32 {
    map_measurements(series)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
        .unwrap_or_default()
}

pub async fn get_weekly_stats(base_url: &str) -> WeeklyStats {
    let sensors_raw_data = get_sensor_raw_data(base_url).await;

    let locations: Vec<LocationMinMax> = sensors_raw_data
        .iter()
        .map(|x| {
            let sensor_data = parse_json::<SensorData>(&x.json);
            let data = sensor_data.data;
            LocationMinMax {
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
            }
        })
        .collect();

    WeeklyStats {
        start: locations
            .iter()
            .min_by_key(|x| x.start_date)
            .unwrap()
            .start_date,
        end: locations
            .iter()
            .max_by_key(|x| x.end_date)
            .unwrap()
            .end_date,
        locations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        static BASE_URL: &str = "https://trzebnica.aqi.eco/pl";

        get_weekly_stats(BASE_URL).await;
        assert!(true);
    }
}

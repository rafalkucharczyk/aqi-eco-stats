mod json_structs;
use crate::json_structs::deserializable::{parse_json, Location, SensorData};
use crate::json_structs::serializable::{LocationMinMax, MeasurementMinMax, WeeklyStats};

use std::cmp::Ordering;

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

async fn get_sensor_data(base_url: &str) -> Vec<String> {
    let tasks = get_locations(base_url).await.into_iter().map(|x| {
        let url = String::from(base_url);
        tokio::spawn(async move {
            get_url_text(
                format!(
                    "{}{}/graph_data.json?type=pm&range=week&ma_h=24",
                    url, x.path
                )
                .as_str(),
            )
            .await
            .unwrap()
        })
    });

    futures::future::join_all(tasks)
        .await
        .iter()
        .map(|x| String::from(x.as_ref().unwrap()))
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
    let sensors_data = get_sensor_data(base_url).await;

    let locations: Vec<LocationMinMax> = sensors_data
        .iter()
        .map(|x| {
            let data = parse_json::<SensorData>(x).data;
            LocationMinMax {
                name: String::from(""), // TODO
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
        start: 0, // TODO
        end: 0,   // TODO
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

mod json_structs;

use std::cmp::Ordering;

use futures;
use reqwest;

use crate::json_structs::deserializable::{parse_json, Location, SensorData};

static BASE_URL: &'static str = "https://trzebnica.aqi.eco/pl";

async fn get_url_text(url: &str) -> Result<String, reqwest::Error> {
    println!("{}", url);
    let body = reqwest::get(url).await?.text().await?;

    Ok(body)
}

async fn get_locations() -> Vec<Location> {
    let data = get_url_text(format!("{}/map/data.json", BASE_URL).as_str())
        .await
        .unwrap_or_default();
    parse_json::<Vec<Location>>(&data)
}

async fn get_sensor_data() -> Vec<String> {
    let tasks = get_locations().await.into_iter().map(|x| {
        tokio::spawn(async move {
            get_url_text(
                format!(
                    "{}{}/graph_data.json?type=pm&range=week&ma_h=24",
                    BASE_URL, x.path
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

pub async fn get_data() -> Option<f32> {
    let sensors_data = get_sensor_data().await;

    parse_json::<SensorData>(&sensors_data.first().unwrap())
        .data
        .pm25
        .into_iter()
        .filter(|x| x.1.is_some())
        .map(|x| x.1.unwrap())
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        get_data().await;
        assert!(true);
    }
}

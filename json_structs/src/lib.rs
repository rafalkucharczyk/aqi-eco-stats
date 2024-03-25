pub mod remote_data {
    use std::collections::HashMap;

    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct Location {
        pub path: String,
        pub description: String,
    }

    #[derive(Deserialize, Debug, Default, PartialEq)]
    pub struct ParticlesData {
        pub pm10: HashMap<u32, Option<f32>>,
        pub pm25: HashMap<u32, Option<f32>>,
    }

    #[derive(Deserialize, Debug, Default, PartialEq)]
    pub struct SensorData {
        pub start: u32,
        pub end: u32,
        pub data: ParticlesData,
    }

    pub fn parse_json<'a, T>(json: &'a str) -> T
    where
        T: Deserialize<'a> + Default,
    {
        let data: T = serde_json::from_str(json).unwrap_or_default();
        data
    }
}

pub mod output {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
    pub struct MeasurementMinMax {
        pub min: f32,
        pub max: f32,
    }

    #[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
    pub struct LocationMinMax {
        pub name: String,
        pub start_date: u32,
        pub end_date: u32,
        pub pm25: MeasurementMinMax,
        pub pm10: MeasurementMinMax,
    }

    #[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
    pub struct WeeklyStats {
        pub start: u32,
        pub end: u32,
        pub locations: Vec<LocationMinMax>,
    }
}

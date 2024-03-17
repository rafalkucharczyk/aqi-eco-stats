pub mod deserializable {
    use std::collections::HashMap;

    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct Location {
        pub path: String,
    }
    #[derive(Deserialize, Debug, Default)]
    pub struct ParticlesData {
        pub pm10: HashMap<u32, Option<f32>>,
        pub pm25: HashMap<u32, Option<f32>>,
    }

    #[derive(Deserialize, Debug, Default)]
    pub struct SensorData {
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

pub mod serializable {
    use serde::Serialize;

    #[derive(Serialize, Debug, Default)]
    pub struct MeasurementMinMax {
        pub min: f32,
        pub max: f32,
    }

    #[derive(Serialize, Debug, Default)]
    pub struct LocationMinMax {
        pub name: String,
        pub pm25: MeasurementMinMax,
        pub pm10: MeasurementMinMax,
    }

    #[derive(Serialize, Debug, Default)]
    pub struct WeeklyStats {
        pub start: u32,
        pub end: u32,
        pub locations: Vec<LocationMinMax>,
    }
}

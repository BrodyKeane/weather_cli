use serde_json::Value;
use reqwest::{Url, blocking};
use std::error::Error;

use crate::arg_parser::{Config, Timeframe, Unit};


pub fn request_weather(config: &Config) -> Result<Value, Box<dyn Error>> {
    let api_key = "1d2bbf22052ec79487d26582e49430a3".to_string();
    
    let lat = config.get_lat().to_string();
    let lon = config.get_lon().to_string();
    let unit = match config.unit {
        Unit::F => "imperial".to_string(),
        Unit::C => "metric".to_string(),
    };

    let mut url = Url::parse_with_params(
        &format!("https://api.openweathermap.org/"), &[
            ("lat", lat),
            ("lon", lon),
            ("units", unit),
            ("appid", api_key),
        ]
    )?;

    match config.timeframe {
        Timeframe::Current => url.set_path("data/2.5/weather"),
        Timeframe::Hourly => {
            url.set_path("data/2.5/forecast");
            url.query_pairs_mut().append_pair("cnt", "8");
        },
        Timeframe::Daily => {
            url.set_path("data/2.5/forecast");
            url.query_pairs_mut().append_pair("cnt", "5");
        },
    };
    let client = blocking::Client::new();
    let response = client.get(url).send()?;
    let data = response.text()?;
    let data: Value = serde_json::from_str(&data)?;
    Ok(data)
}


pub struct WeatherObject {
    pub description: String,
    pub temp: f64,
    pub wind_speed: f64,
    pub time: String,
}

impl WeatherObject {
    pub fn build_all(data: Value) -> Vec<Self> {
        if let None = data.get("list") { 
            return vec![Self::build(&data)]
        }
        let data_groups = data["list"].as_array().unwrap().clone();
        data_groups.iter().map(|x| Self::build(x)).collect()
    }

    fn build(data: &Value) -> WeatherObject {
        let description = data["weather"][0]["main"].to_string();
        let temp = data["main"]["temp"].as_f64().unwrap();
        let wind_speed = data["wind"]["speed"].as_f64().unwrap();
        let time = match data.get("dt_txt") {
            None => "Now".to_string(),
            Some(_) => {
                let verbose_time = data["dt_txt"].to_string();
                verbose_time[12..=16].to_string()
            }
        };
        WeatherObject{
            description,
            temp,
            wind_speed,
            time,
        }
    }
}
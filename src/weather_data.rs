use serde_json::Value;
use reqwest::{Url, blocking};
use std::error::Error;

use crate::arg_parser::{Config, Timeframe, Unit};

pub mod weather_api {
    use super::*;

    pub fn request_weather(config: &Config) -> Result<Value, Box<dyn Error>> {
        let api_key = "1d2bbf22052ec79487d26582e49430a3".to_string();
        
        let lat = config.get_lat().to_string();
        let lon = config.get_lon().to_string();
        let unit = match config.unit {
            Unit::F => "imperial".to_string(),
            Unit::C => "metric".to_string(),
        };

        let url = match config.timeframe {
            Timeframe::Current => {
                Url::parse_with_params(
                    &format!("https://api.openweathermap.org/data/2.5/weather"), &[
                        ("lat", lat),
                        ("lon", lon),
                        ("units", unit),
                        ("appid", api_key),
                    ]
                )?
            }
            Timeframe::Hourly => {
                Url::parse_with_params(
                    &format!("http://api.openweathermap.org/data/2.5/forecast?"), &[
                        ("lat", lat),
                        ("lon", lon),
                        ("units", unit),
                        ("cnt", "8".to_string()),
                        ("appid", api_key),
                    ]
                )?
            }
            Timeframe::Daily => {
                Url::parse_with_params(
                        &format!("http://api.openweathermap.org/data/2.5/forecast?"), &[
                        ("lat", lat),
                        ("lon", lon),
                        ("units", unit),
                        ("cnt", "5".to_string()),
                        ("appid", api_key),
                    ]                    
                )?
            }
        };
        println!("{}", url);
        let client = blocking::Client::new();
        let response = client.get(url).send()?;
        let data = response.text()?;
        let json_data: Value = serde_json::from_str(&data)?;

        println!("{}", json_data);
        Ok(json_data)
    }
}
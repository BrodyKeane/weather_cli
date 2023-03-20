use serde_json::Value;
use reqwest::{Url, blocking};
use std::error::Error;

use crate::arg_parser::{Config, Timeframe, Unit};

pub use weather_data::{request_weather, WeatherObject, print_weather};

pub mod weather_data{
    use super::*;

    pub fn request_weather(config: &Config) -> Result<Value, Box<dyn Error>> {
        let url = url_builder(&config)?;
        let client = blocking::Client::new();
        let response = client.get(url).send()?;
        let text = response.text()?;
        let data: Value = serde_json::from_str(&text)?;

        Ok(data)
    }

    fn url_builder(config: &Config) -> Result<Url, Box<dyn Error>> {
        let api_key = config.get_weather_key().unwrap().to_string();
        let lat = config.get_lat().unwrap().to_string();
        let lon = config.get_lon().unwrap().to_string();
        let unit = match config.get_unit() {
            Some(Unit::F) => "imperial".to_string(),
            Some(Unit::C) => "metric".to_string(),
            None => "metric".to_string(),
        };

        let mut url = Url::parse_with_params( 
            "https://api.openweathermap.org/", &[
                ("lat", lat),
                ("lon", lon),
                ("units", unit),
                ("appid", api_key),
            ]
        )?;

        match config.get_timeframe() {
            None => url.set_path("data/2.5/weather"),
            Some(Timeframe::Current) => url.set_path("data/2.5/weather"),
            Some(Timeframe::Hourly) => {
                url.set_path("data/2.5/forecast");
                url.query_pairs_mut().append_pair("cnt", "8");
            },
            Some(Timeframe::Daily) => {
                url.set_path("data/2.5/forecast");
                url.query_pairs_mut().append_pair("cnt", "32");
            },
        };
        Ok(url)
    }

    #[derive(Clone)]
    pub struct WeatherObject {
        pub description: String,
        pub temp: f64,
        pub wind_speed: f64,
        pub date: String,
        pub time: String,
    }

    impl WeatherObject {
        pub fn build_all(data: Value, config: Config) -> Vec<Self> {
            if let None = data.get("list") { 
                return vec![Self::build(&data)]
            }
            let data_groups = data["list"].as_array().unwrap();
            match config.timeframe.unwrap() {
                Timeframe::Current => vec![Self::build(&data)],
                Timeframe::Hourly => data_groups.iter().map(|x| Self::build(x)).collect(),
                Timeframe::Daily => {
                    let weather_objects: Vec<Self> = data_groups.iter().map(|x| Self::build(x)).collect();
                    let mut daily_objects = Vec::new();
                    for (i, object) in weather_objects.iter().enumerate() {
                        if i == 0 || (i + 1) % 8 == 0  {
                            daily_objects.push(object.clone());
                        }
                    }
                    daily_objects
                },
            }
        }

        fn build(data: &Value) -> WeatherObject {
            let description = data["weather"][0]["main"].to_string();
            let description = description[1..description.len()-1].to_string();
            let temp = data["main"]["temp"].as_f64().unwrap();
            let wind_speed = data["wind"]["speed"].as_f64().unwrap();
            let mut date = String::new();
            
            let mut time = String::new();
            match data.get("dt_txt") {
                None => {
                    date.push_str("Today");
                    time.push_str("Now");
                },
                Some(_) => {
                    let dt = data["dt_txt"].to_string();
                    date.push_str(&dt[6..=10]);
                    time.push_str(&dt[12..=16]);
                },
            };
            WeatherObject{
                description,
                temp,
                wind_speed,
                time,
                date,
            }
        }
    }

    pub fn print_weather(weather_objects: &Vec<WeatherObject>) {
        let line = "+-------------------------------------------------+";  
        println!("{line}");
        println!("|  Date   |  Time   |  Temp   | Weather |  Wind   |"); 
        for object in weather_objects {
            println!("{line}");
            println!(
                "| {date:<7} | {time:<7} | {temp:<7} | {description:<7} | {wind_speed:<7} |",
                date=object.date, time=object.time, temp=object.temp,
                description=object.description, wind_speed=object.wind_speed,
            );
        }
        println!("{line}");
    }
}
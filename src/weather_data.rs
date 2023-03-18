use serde_json::Value;
use reqwest::{Url, blocking};
use std::error::Error;

use crate::arg_parser::{Config, Timeframe, Unit};


pub fn request_weather(config: &Config) -> Result<Value, Box<dyn Error>> {
    let url = url_builder(&config)?;
    let client = blocking::Client::new();
    let response = client.get(url).send()?;
    let data = response.text()?;
    let data: Value = serde_json::from_str(&data)?;
    Ok(data)
}

fn url_builder(config: &Config) -> Result<Url, Box<dyn Error>> {
    let api_key = "1d2bbf22052ec79487d26582e49430a3".to_string();
    let lat = config.get_lat().to_string();
    let lon = config.get_lon().to_string();
    let unit = match config.unit {
        Unit::F => "imperial".to_string(),
        Unit::C => "metric".to_string(),
    };

    let mut url = Url::parse_with_params( 
        "https://api.openweathermap.org/", &[
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
            url.query_pairs_mut().append_pair("cnt", "32");
        },
    };
    Ok(url)
}

pub struct WeatherObject {
    pub description: String,
    pub temp: f64,
    pub wind_speed: f64,
    pub date: String,
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
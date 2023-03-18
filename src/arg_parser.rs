use std::io::{self, Write, Read};
use serde_json::{Value, json};
use std::fs::File;
use geocoding::{Opencage, Point, Forward};

use crate::api_keys::ApiKeys;

pub use arg_parser::{Config, Timeframe, Unit};

pub mod arg_parser {
    use super::*;

    pub struct Config {
        pub timeframe: Timeframe,
        pub unit: Unit,
        coords: Coords,
        keys: ApiKeys
    }

    pub enum Timeframe {
        Current,
        Hourly,
        Daily,
    }

    pub enum Unit {
        F,
        C,
    }

    struct Coords {
        lat: String,
        lon: String,
    }

    impl Config {
        pub fn build(arg: Option<String>) -> Result<Config, String> {
            let timeframe = Self::match_timeframe(arg)?;
            
            let path = "config.json";
            let mut file = std::fs::File::open(&path).unwrap();
            let mut contents = String::new();
            if let Err(_) = file.read_to_string(&mut contents) {
                return Err("An unknown error occurred while trying to read your config file".to_string());
            }
            let json_obj: Value = match serde_json::from_str(&contents) {
                Ok(value) => value,
                Err(err) => return Err(err.to_string()),
            };

            let keys = ApiKeys::request_keys(&json_obj);
            let unit = Self::request_unit(&json_obj);
            let coords = Self::request_coords(&json_obj);

            let config = Config{ timeframe, unit, coords, keys };
            config.update_json(&path);

            Ok(config)
        }

        fn match_timeframe(arg: Option<String>) -> Result<Timeframe, String> {
            match arg {
                None => Ok(Timeframe::Current),
                Some(val) => match val.as_str() {
                    "current" => Ok(Timeframe::Current),
                    "hourly" => Ok(Timeframe::Hourly),
                    "daily" => Ok(Timeframe::Daily),
                    input => return Err(format!("Failed to parse input: {}", input).into()),
                }
            }
        }

        
        fn request_unit(json_obj: &Value) -> Unit {
            if let Some(val) = json_obj["unit"].as_str() {
                match val.trim().to_uppercase().as_str() {
                    "F" => return Unit::F,
                    "C" => return Unit::C,
                    _ => (),               
                }
            } 
            loop {
                println!("Please input your preferred unit. (F or C):");
                let mut input = String::new();

                if let Err(error) = io::stdin().read_line(&mut input) {
                    println!("Error reading input: {:?}", error);
                    continue
                }

                match input.trim().to_uppercase().as_str() {
                    "F" => return Unit::F,
                    "C" => return Unit::C,
                    _ => println!("Input didn't match values: (F or C)."),
                }
            }
        }

        fn request_coords(json_obj: &Value) -> Coords {
            if let Some(lat) = json_obj["lat"].as_str() {
                if let Some(lon) = json_obj["lon"].as_str() {
                    return Coords{lat: lat.to_string(), lon: lon.to_string()}
                }
            }

            let api_key = "2l14a2ef217c849719cbf6d2533db560a".to_string();
            let geocoder = Opencage::new(api_key); 
            let mut location = String::new();
            loop {
                println!("Enter your location using any standard format");
                if let Err(error) = io::stdin().read_line(&mut location) {
                    println!("Error reading unput: {:?}", error);
                    continue
                }

                let location = location.trim().to_string();
                let result: Vec<Point<f64>> = match geocoder.forward(&location) { 
                    Ok(val) => val,
                    Err(error) => {
                        println!("Error: {}", error);
                        continue
                    }
                };
            
                let lon = &result[0].x();
                let lat = &result[0].y();
                    return Coords {lat: lat.to_string(), lon: lon.to_string()}
            }
        }

        fn update_json(&self, path: &str) {
            let data = json!({
                "unit": self.get_unit(),
                "lat": self.get_lat(),
                "lon": self.get_lon(),
                "weather_key": self.get_weather_key(),
                "location_key": self.get_location_key(),
            });

            let json_string = serde_json::to_string(&data).unwrap();
            let mut file = File::create(path).unwrap();
            file.write_all(json_string.as_bytes()).unwrap();
        }     

        pub fn get_unit(&self) -> &str{
            match self.unit {
                Unit::F => "F",
                Unit::C => "C",
            }
        }

        pub fn get_lat(&self) -> &String {
            &self.coords.lat
        }

        pub fn get_lon(&self) -> &String {
            &self.coords.lon
        }

        pub fn get_weather_key(&self) -> &String {
            &self.keys.weather_key
        }

        pub fn get_location_key(&self) -> &String {
            &self.keys.location_key
        }
    }
}
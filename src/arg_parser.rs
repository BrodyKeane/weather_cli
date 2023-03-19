use std::fs::{self, File};
use std::error::Error;
use std::io::{self, Write, Read};
use geocoding::{Opencage, Point, Forward};
use serde::{Serialize, Deserialize};

use crate::api_keys::ApiKeys;

pub use arg_parser::{Config, Timeframe, Unit};

pub mod arg_parser {
    use super::*;

    #[derive(Serialize, Deserialize)]
    pub struct Config {
        pub timeframe: Option<Timeframe>,
        pub unit: Option<Unit>,
        coords: Option<Coords>,
        keys: Option<ApiKeys>,
    }

    #[derive(Serialize, Deserialize)]
    pub enum Timeframe {
        Current,
        Hourly,
        Daily,
    }

    #[derive(Serialize, Deserialize)]
    pub enum Unit {
        F,
        C,
    }

    #[derive(Serialize, Deserialize)]
    struct Coords {
        lat: String,
        lon: String,
    }

    impl Config {
        pub fn build(arg: Option<String>) -> Result<Config, Box<dyn Error>> {
            let path = "config.json";
            let mut file = match fs::File::open(&path) {
                Ok(f) => f,
                Err(_) => {
                    fs::write(path, "{}")?;
                    fs::File::open(&path)?
                },
            };
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            let mut config: Config = serde_json::from_str(&contents)?;

            config.set_timeframe(arg)?;
            config.set_keys();
            config.set_unit();
            config.set_coords();

            config.update_json(&path)?;

            Ok(config)
        }

        fn update_json(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
            let json = serde_json::to_string(&self)?;
            let mut file = File::create(path)?;
            file.write_all(json.as_bytes())?;
            Ok(())
        }     

        fn set_timeframe(&mut self, arg: Option<String>) -> Result<(), String> {
            self.timeframe = match arg {
                None => Some(Timeframe::Current),
                Some(val) => match val.as_str() {
                    "current" => Some(Timeframe::Current),
                    "hourly" => Some(Timeframe::Hourly),
                    "daily" => Some(Timeframe::Daily),
                    input => return Err(format!("Failed to parse input: {}", input).into()),
                }
            };
            Ok(())
        }

        fn set_keys(&mut self) {
            self.keys = Some(ApiKeys::request_keys(self));
        }
        
        fn set_unit(&mut self) {
            if let Some(_) = self.unit {
                return
            } 
            loop {
                println!("Please input your preferred unit. (F or C):");
                let mut input = String::new();

                if let Err(error) = io::stdin().read_line(&mut input) {
                    println!("Error reading input: {:?}", error);
                    continue
                }

                let unit = match input.trim().to_uppercase().as_str() {
                    "F" => Unit::F,
                    "C" => Unit::C,
                    _ => {
                        println!("Input didn't match values: (F or C).");
                        continue
                    },
                };

                self.unit = Some(unit);
                break
            }
        }

        fn set_coords(&mut self) {
            if let Some(_) = self.coords {
                return
            }

            let key = self.get_location_key().unwrap().to_string();
            let geocoder = Opencage::new(key); 
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

                
                self.coords = Some(
                    Coords{
                        lat: lat.to_string(),
                        lon: lon.to_string()
                    }
                );
                return 
            }
        }

        pub fn get_timeframe(&self) -> &Option<Timeframe> {
            &self.timeframe
        }
        
        pub fn get_unit(&self) -> &Option<Unit> {
            &self.unit
        }

        pub fn get_lat(&self) -> Option<&String> {
            match &self.coords {
                Some(coords) => Some(&coords.lat),
                None => None,
            }
        }

        pub fn get_lon(&self) -> Option<&String> {
            match &self.coords {
                Some(coords) => Some(&coords.lon),
                None => None,
            }
        }

        pub fn get_weather_key(&self) -> Option<&String> {
            match &self.keys {
                Some(keys) => Some(&keys.weather_key),
                None => None,
            }
        }

        pub fn get_location_key(&self) -> Option<&String> {
            match &self.keys {
                Some(keys) => Some(&keys.location_key),
                None => None,
            }
        }
    }
}
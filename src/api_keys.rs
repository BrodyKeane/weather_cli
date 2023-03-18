use std::error::Error;
use std::io;
use serde_json::Value;
use reqwest::{self, blocking::Client, Url};

pub use api_keys::ApiKeys;

pub mod api_keys{
    use super::*;

    pub struct ApiKeys {
        pub weather_key: String,
        pub location_key: String, 
    }

    impl ApiKeys {
        pub fn request_keys(json_obj: &Value) -> ApiKeys {
            let weather_key = match json_obj.get("weather_key") {
                Some(key) => key.to_string(),
                None => Self::request_weather_key(),
            };

            let location_key = match json_obj.get("location_key") {
                Some(key) => key.to_string(),
                None => Self::request_location_key(),
            };
            ApiKeys {weather_key, location_key}
        }

        fn request_weather_key() -> String {
            let url = "https://home.openweathermap.org/users/sign_up";
            loop {
                println!("\nEnter get your api key from: {url}");

                let mut key = String::new();
                if let Err(error) = io::stdin().read_line(&mut key) {
                    println!("Error reading input: {:?}", error);
                    continue
                }

                let key = key.trim().to_string();
                match Self::verify_weather_key(&key) {
                    Ok(()) => return key,
                    Err(error) => {
                        println!("{}", error);
                        continue
                    },
                };
            }
        }

        fn verify_weather_key(key: &String) -> Result<(), Box<dyn Error>> {
            let url = Url::parse_with_params(
                "https://api.openweathermap.org/data/2.5/weather", &[
                    ("lat", "37"),
                    ("lon", "95"),
                    ("appid", key),
                ]
            )?;
            Self::verify_key(url)
        }

        fn request_location_key() -> String {
            let url = "https://opencagedata.com/users/sign_up";
            loop {
                println!("\nEnter your other api key from: {url}");

                let mut key = String::new();
                if let Err(error) = io::stdin().read_line(&mut key) {
                    println!("Error reading input: {:?}", error);
                    continue
                }

                let key = key.trim().to_string();
                match Self::verify_location_key(&key) {
                    Ok(()) => return key,
                    Err(error) => {
                        println!("Unable to validate key: {}", error);
                        println!();
                        continue
                    },
                };
            }
        }

        fn verify_location_key(key: &String) -> Result<(), Box<dyn Error>> {
            let url = Url::parse_with_params(
                "https://api.opencagedata.com/geocode/v1/json", &[
                    ("q", "United States"),
                    ("key", key),
                ]
            )?;
            Self::verify_key(url)
        }

        fn verify_key(url: Url) -> Result<(), Box<dyn Error>> {
            let client = Client::new();
            let response = client.get(url).send()?;
            
            match response.status().is_success() {
                true => {
                    println!("API key validated");
                    Ok(())
                }
                false => { 
                    Err("Failed to validate key.\nKeys can take up to an hour to become active.".into())
                }
            }
        }
    }
}
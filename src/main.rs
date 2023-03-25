use core::time;
use std::{
    error::Error,
    process,
    env,
    fs::{self, File},
    io::{self, Write},
    net::TcpStream,
};

use serde_json::{self, Value, json};
use reqwest::{self, blocking, Url};
use geolocation;

enum Timeframe {
    Current,
    Hourly,
    Daily,
}

struct Coords {
    lat: String,
    lon: String,
}

fn main() {
    run().unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    }); 
}

fn run() -> Result<(), Box<dyn Error>> {
    let timeframe = get_timeframe()?;
    let coords = get_coords();

    let config_path = "config.json";
    let mut config = get_config(config_path)?;

    if let None = config.get("weather_key") {
        request_weather_key(&mut config)?;
   };

    if let None = config.get("unit") {
        request_unit(&mut config);
    }

    let url = build_url(&config, coords, &timeframe)?;
    let weather_objects = get_weather_data(url, &timeframe)?;

    print_weather(weather_objects);
    update_json(config, config_path)?;

    Ok(())
}

fn get_timeframe() -> Result<Timeframe, String> {
    match env::args().skip(1).next() {
        None => Ok(Timeframe::Current),
        Some(arg) => match arg.as_str().trim() {
            "current" => Ok(Timeframe::Current),
            "hourly" => Ok(Timeframe::Hourly),
            "daily" => Ok(Timeframe::Daily),
            _ => Err(format!("{} is not a valid argument please input (current, hourly, or daily)", arg).into())
        }
    }
}

fn get_config(config_path: &str) -> Result<Value, Box<dyn Error>>{
    let config_string = match fs::read_to_string(config_path) {
        Ok(string) => string,
        Err(_) => "{}".to_string(),
    };
    Ok(serde_json::from_str(&config_string)?)
}

fn request_weather_key(config: &mut Value) -> Result<(), Box<dyn Error>> {
    let url = "https://home.openweathermap.org/users/sign_up";
    let weather_key = loop {
        println!("\nEnter your other api key from: {url}");

        let mut key = String::new();
        if let Err(error) = io::stdin().read_line(&mut key) {
            println!("Error reading input: {:?}", error);
            continue
        }
        key = key.trim().to_string();

        let test_url = Url::parse_with_params(
            "https://api.openweathermap.org/data/2.5/weather", &[
                ("lat", "37"),
                ("lon", "95"),
                ("appid", &key),
            ]
        )?;

        let key_is_valid = blocking::Client::new()
            .get(test_url)
            .send()?
            .status()
            .is_success();

        match key_is_valid {
            true => {
                println!("\nKey Validated");
                break key
            },
            false => eprintln!("\nFailed to validate key.\nKeys can take up to an hour to become active."),
        }
    };
    config
        .as_object_mut()
        .unwrap()
        .insert("weather_key".to_string(), json!(weather_key));
    Ok(())
}

fn request_unit(config: &mut Value) {
    let unit = loop {
        println!("\nPlease input your preferred unit. (imperial or metric):");
        let mut input = String::new();

        if let Err(error) = io::stdin().read_line(&mut input) {
            eprintln!("Error reading input: {:?}", error);
            continue
        }

        match input.trim().to_lowercase().as_str() {
            "imperial" => break "imperial",
            "metric" => break "metric",
            _ => {
                eprintln!("Input didn't match values: (imperial or metric).");
                continue
            },
        };
    };

    config
        .as_object_mut()
        .unwrap()
        .insert("unit".to_string(), json!(unit));
}

fn get_coords() -> Coords {
    let stream = TcpStream::connect("example.com:80").unwrap();
    let addr = stream.peer_addr().unwrap().to_string();
    let addr_parts: Vec<&str> = addr.split(":").collect();
    let ip = addr_parts[0];
    let info = geolocation::find(&ip).unwrap();

    Coords{ 
        lat: info.latitude,
        lon: info.longitude
    }
}

fn build_url(config: &Value, coords: Coords, timeframe: &Timeframe)
    -> Result<Url, Box<dyn Error>> {

    let unit = config["unit"].to_string().replace("\"", "");
    let key = config["weather_key"].to_string().replace("\"", "");

    let mut url = Url::parse_with_params(
        "https://api.openweathermap.org/", &[
            ("lat", coords.lat),
            ("lon", coords.lon),
            ("units", unit),
            ("appid", key),
        ]
    )?;

    match timeframe {
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

fn get_weather_data(url: Url, timeframe: &Timeframe) -> Result<Vec<Value>, Box<dyn Error>> {
    let weather_data: Value = serde_json::from_str(
        &blocking::Client::new()
        .get(url.to_string())
        .send()?
        .text()?
    )?;
    match timeframe {
        Timeframe::Current => Ok(vec![weather_data]),
        Timeframe::Hourly => Ok(weather_data["list"].as_array().unwrap().to_vec()),
        Timeframe::Daily => {
            let all_objects = weather_data["list"].as_array().unwrap();
            let mut daily_objects = Vec::new();
            for (i, object) in all_objects.iter().enumerate() {
                if i == 0 || (i + 1) % 8 == 0  {
                    daily_objects.push(object.clone());
                }
            }
            Ok(daily_objects)
        }
    }
}

fn print_weather(weather_objects: Vec<Value>) {
    let line = "+-------------------------------------------------+";  
    println!("{line}");
    println!("|  Date   |  Time   |  Temp   | Weather |  Wind   |"); 
    for object in weather_objects {
        let mut date = String::new();
        let mut time = String::new();
        match object.get("dt_txt") {
            None => {
                date.push_str("Today");
                time.push_str("Now");
            },
            Some(_) => {
                let dt = object["dt_txt"].to_string();
                date.push_str(&dt[6..=10]);
                time.push_str(&dt[12..=16]);
            },
        };

        let mut description = object["weather"][0]["main"].to_string();
        description = description[1..description.len()-1].to_string();
            
        println!("{line}");
        println!(
            "| {date:<7} | {time:<7} | {temp:<7} | {description:<7} | {wind_speed:<7} |",
            date=date,
            time=time,
            temp=object["main"]["temp"].as_f64().unwrap(),
            description=description,
            wind_speed=object["wind"]["speed"].as_f64().unwrap(),
        );
    }
    println!("{line}");
}

fn update_json(config: Value, config_path: &str) -> Result<(), Box<dyn Error>> {
    let json_string = serde_json::to_string(&config)?;
    let mut file = File::create(config_path)?;
    file.write_all(json_string.as_bytes())?;
    Ok(())
}
use std::env;
use std:: process;

mod arg_parser;
use arg_parser::Config;

mod weather_data;
use weather_data::weather_api;

fn main() {
    let arg = env::args().skip(1).next();
    let config = Config::build(arg).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });  

    let _data = weather_api::request_weather(&config).unwrap_or_else(|err| {
        eprintln!("Problem retrieving weather data: {err}");
        process::exit(1);
    });
}



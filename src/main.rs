use std::env;
use std:: process;

mod arg_parser;
use arg_parser::Config;

mod weather_data;
use weather_data::WeatherObject;

mod api_keys;

fn main() {
    let arg = env::args().skip(1).next();
    let config = Config::build(arg).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });  

    let data = weather_data::request_weather(&config).unwrap_or_else(|err| {
        eprintln!("Problem retrieving weather data: {err}");
        process::exit(1);
    });

    let weather_objects = WeatherObject::build_all(data);
    weather_data::print_weather(&weather_objects);
}

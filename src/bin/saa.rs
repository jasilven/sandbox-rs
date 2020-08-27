use anyhow::Result;
use chrono::prelude::*;
use reqwest;
use serde::{Deserialize, Serialize};

static KEY: &str = include_str!("ow-apikey.txt");

#[derive(Serialize, Deserialize, Debug)]
struct Weather {
    description: String,
    icon: String,
    id: usize,
    main: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Wind {
    deg: usize,
    speed: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Main {
    feels_like: f64,
    humidity: usize,
    pressure: usize,
    temp: f64,
    temp_min: f64,
    temp_max: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Clouds {
    all: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct WeatherResp {
    name: String,
    weather: Vec<Weather>,
    wind: Wind,
    main: Main,
    clouds: Clouds,
    timezone: isize,
}

impl ToString for WeatherResp {
    fn to_string(&self) -> String {
        let mut desc = String::from("");
        let mut icons = String::from("");

        for w in self.weather.iter() {
            desc.push_str(&format!("{},", w.description));
            match w.icon.as_str() {
                "01d" | "01n" => icons.push('🌣'),
                "02d" | "02n " => icons.push('🌤'),
                "03d" | "03n" => icons.push('🌥'),
                "04d" | "04n" => icons.push('☁'),
                "09d" | "10d" | "09n" | "10n" => icons.push('🌧'),
                "05d" | "06d" | "05n" | "06n" => icons.push('🌧'),
                "11d" | "11n" => icons.push('🌩'),
                "13d" | "13n" => icons.push('🌨'),
                _ => icons.push('-'),
            };
        }

        let time =
            match Utc::now().checked_add_signed(chrono::Duration::seconds(self.timezone as i64)) {
                Some(t) => t.format("%k:%M").to_string(),
                None => "     ".to_string(),
            };

        format!(
            "{:>15}: {} {}  {} ℃ , wind {} m/s, {}",
            self.name,
            time,
            icons.trim_end_matches(' '),
            self.main.temp as usize,
            self.wind.speed as usize,
            desc.trim_end_matches(","),
        )
    }
}

async fn get_weather(city: &str) -> Result<WeatherResp> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, KEY
    );
    let result = reqwest::get(&url).await?.json::<WeatherResp>().await?;
    Ok(result)
}

#[tokio::main]
async fn main() -> Result<()> {
    for city in std::env::args().skip(1) {
        let weather = get_weather(&city).await?;
        println!("{}", &weather.to_string());
    }

    Ok(())
}

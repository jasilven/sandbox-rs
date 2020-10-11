#![warn(rust_2018_idioms)]
use anyhow::Result;
use reqwest;
use reqwest::blocking;
use serde_json::Value;

pub(crate) static URL: &str = "https://api.chucknorris.io/jokes/random";

fn main() -> Result<()> {
    let resp = blocking::get(URL)?.json::<Value>()?;

    dbg!(&resp);
    println!("{}", resp["value"]);

    Ok(())
}

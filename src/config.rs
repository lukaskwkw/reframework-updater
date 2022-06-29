use std::{collections::HashMap, error::Error, str::FromStr};

use serde::{Deserialize, Serialize};
use toml::{value::Datetime, Value};
use crate::FromValue::FromValue;

#[derive(Serialize, Deserialize, Debug)]
enum Runtime {
    OpenVR,
    OpenXR,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    location: Option<String>,
    versions: Option<Vec<Datetime>>,
    nextgen: Option<bool>,
    runtime: Option<Runtime>,
    runArgs: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Main {
    sources: Option<Vec<String>>,
    autoupdate: Option<bool>,
}

pub fn serialize() {
    let config = Config {
        location: Some(String::from("c:/steam/games/RE7")),
        versions: Some(vec![Datetime::from_str("2022-06-26T14:18:41Z").unwrap()]),
        nextgen: Some(false),
        runtime: Some(Runtime::OpenVR),
        runArgs: None,
    };

    let toml = toml::to_string(&config).unwrap();
    let formatted = format!("[RE7]\n{}", toml);
    println!("{}", formatted);
}

pub fn deserialize() -> Result<(Main, HashMap<String, Config>), Box<dyn Error>> {
    let value = r#"
        [main]
        sources = ["normal", "nightly"]
        autoupdate = true

        [RE2]
        location = "D:/steam/games/RE7"
        latest = [2022-06-21T14:18:41Z]
        nextgen = true
        runtime = "OpenXR"

        [RE7]
        location = "D:/steam/games/RE7"
        latest = [2022-06-21T14:18:41Z]
        [RE12]
        location = "D:/steam/games/RE12"
        latest = [2022-06-21T14:18:41Z]
        latawiec = true
    "#
    .parse::<Value>()?;

    let table = value.as_table().ok_or("No table fields!")?;

    let (_, main_value) = table.iter().find(|(s, v)| s == &"main").ok_or("Main Not Found")?;

    let main: Main = Value::from_value(main_value.to_owned())?;

    println!("{:?}", main);

    let games = table
        .iter()
        .filter_map(|(s, v)| {
            if s == "main" {
                return None;
            }
            let config = match Value::from_value(v.to_owned()) {
                Ok(it) => it,
                Err(err) => {
                    eprintln!("games toml::from_value error: {}", err);
                    return None;
                }
            };
            Some((s.to_string(), config))
        })
        .collect::<HashMap<String, Config>>();

    games.iter().for_each(|game| println!("{:?}", game));

    return Ok((main, games));
}

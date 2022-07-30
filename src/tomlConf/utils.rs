use std::collections::HashMap;

use error_stack::{IntoReport, Result, ResultExt};
use toml::Value;

use crate::{tomlConf::configStruct::ErrorLevel, MAX_ZIP_FILES_PER_GAME_CACHE};

use super::{
    configStruct::{ConfigError, GameConfig, Main, REvilConfig},
    FromValue::FromValue,
};

type ConfigResult<T> = Result<T, ConfigError>;

pub fn serialize(config: &REvilConfig) -> ConfigResult<String> {
    let main_table = toml::to_string_pretty(&config.main)
        .report()
        .change_context(ConfigError::Serializer)
        .attach_printable_lazy(|| {
            format!("Error during serialization of main {:?}", &config.main)
        })?;

    let mut games_vec: Vec<_> = config
        .games
        .iter()
        .collect();
        games_vec.sort_by_key(|tuple| tuple.0.to_string());
        
        let config_str = games_vec
        .iter()
        .map(|(key, value)| {
            let config_str = toml::to_string_pretty(&value)
                .report()
                .change_context(ConfigError::Serializer)
                .attach_printable(format!(
                    "err during serialization of key {} value {:?}",
                    key, value
                ))
                .unwrap();
            format!("[{}]\n{}\n", key, config_str)
        })
        .reduce(|acc, config_str| format!("{}{}", acc, config_str));

    let all = format!(
        "[main]\n{}\n{}",
        main_table,
        config_str.ok_or("Reduce error").unwrap()
    );
    Ok(all)
}

pub fn deserialize(content: &str) -> ConfigResult<(Main, HashMap<String, GameConfig>)> {
    // TODO when there will be wrong toml syntax in config file for particular key then it will be treated
    //      like config file error and all config.toml content will be altered with new content
    //      not sure how to handle it differently -> priority very minor, it can be too much hassle I think. Also below I already handle some errors
    let value = content
        .parse::<Value>()
        .report()
        .change_context(ConfigError::Deserializer)?;

    let table = match value.as_table() {
        Some(table) => table,
        None => return Err(ConfigError::Deserializer)?,
    };

    let (_key, main_value) = match table.iter().find(|(s, _v)| s == &"main") {
        Some((key, main_value)) => (key, main_value),
        None => Err(ConfigError::Deserializer)
            .report()
            .attach_printable("Main not found!")?,
    };
    let main = Value::from_value(main_value.to_owned()).unwrap_or_else(|err| {
        eprintln!(
            "Error during deserialization of toml main section {:#?} setting default for main",
            err
        );
        Main {
            max_cache_versions_per_game: Some(MAX_ZIP_FILES_PER_GAME_CACHE),
            errorLevel: Some(ErrorLevel::info),
            ..Main::default()
        }
    });

    let games = table
        .iter()
        .filter_map(|(s, v)| {
            if s == "main" {
                return None;
            }
            let config = match Value::from_value(v.to_owned()) {
                Ok(it) => it,
                Err(err) => {
                    eprintln!("Deserializer error - Reading {} game config {:#?} from toml \n toml::from_value error: {}", s, v, err);
                    return None;
                }
            };
            Some((s.to_string(), config))
        })
        .collect::<HashMap<String, GameConfig>>();

    // games.iter().for_each(|game| println!("{:?}", game));
    Ok((main, games))
}

use std::collections::HashMap;

use error_stack::{IntoReport, Result, ResultExt};
use toml::Value;

use super::{
    configStruct::{ConfigError, GameConfig, Main, REvilConfig},
    FromValue::FromValue,
};

type ConfigResult<T> = Result<T, ConfigError>;

pub fn serialize(config: &REvilConfig) -> ConfigResult<String> {
    let main_table = toml::to_string_pretty(&config.main)
        .report()
        .change_context(ConfigError::SerializerError)
        .attach_printable_lazy(|| {
            format!("Error during serialization of main {:?}", &config.main)
        })?;

    let config_str = config
        .games
        .iter()
        .map(|(key, value)| {
            let config_str = toml::to_string_pretty(&value)
                .report()
                .change_context(ConfigError::SerializerError)
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
    let value = content
        .parse::<Value>()
        .report()
        .change_context(ConfigError::DeserializerError)?;

    let table =match value.as_table() {
        Some(table) => table,
        None => return Err(ConfigError::DeserializerError)?,
    };

    let (_key, main_value) = match table
            .iter()
            .find(|(s, _v)| s == &"main") {
        Some((key, main_value)) => (key, main_value),
        None => Err(ConfigError::DeserializerError).report().attach_printable("Main not found!")?,
    };

    let main: Main = Value::from_value(main_value.to_owned())
        .report()
        .change_context(ConfigError::DeserializerError)?;

    let games = table
        .iter()
        .filter_map(|(s, v)| {
            if s == "main" {
                return None;
            }
            let config = match Value::from_value(v.to_owned()) {
                Ok(it) => it,
                Err(err) => {
                    eprintln!("Deserializer error - Reading {} game config {:#?} from toml \n toml::from_value error: {}", s, v.to_string(), err);
                    return None;
                }
            };
            Some((s.to_string(), config))
        })
        .collect::<HashMap<String, GameConfig>>();

    // games.iter().for_each(|game| println!("{:?}", game));
    Ok((main, games))
}

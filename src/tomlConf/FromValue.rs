use serde::de::DeserializeOwned;
use toml::Value;

pub trait FromValue {
    fn from_value<T>(value: Value) -> Result<T, toml::de::Error>
    where
        T: DeserializeOwned;
}

impl FromValue for Value {
    fn from_value<T>(value: Value) -> Result<T, toml::de::Error>
    where
        T: DeserializeOwned,
    {
        T::deserialize(value)
    }
}

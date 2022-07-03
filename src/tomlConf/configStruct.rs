use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Debug}, error::Error,
};

pub type ConfigResult<T> = Result<T, ConfigError>;

#[derive(Debug)]
pub enum ConfigError {
    ConfigFileError,
    DeserializerError,
    SerializerError,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ConfigError")
    }
}

impl Error for ConfigError {}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ErrorLevel {
    info,
    debug,
    warn,
    error,
    trace
}

impl std::fmt::Display for ErrorLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Runtime {
    OpenVR,
    OpenXR,
}

impl Runtime {
    pub fn as_local_dll(&self) -> String {
        match self {
            Runtime::OpenVR => "openvr_api.dll".to_owned(),
            Runtime::OpenXR => "openxr_loader.dll".to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct GameConfig {
    pub location: Option<String>,
    pub steamId: Option<String>,
    pub versions: Option<Vec<String>>,
    pub nextgen: Option<bool>,
    pub runtime: Option<Runtime>,
    pub runArgs: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Main {
    pub sources: Option<Vec<String>>,
    pub autoupdate: Option<bool>,
    pub steamExePath: Option<String>,
    pub steamGamesIdToSearchFor: Option<Vec<String>>,
    pub errorLevel: Option<ErrorLevel>,
}

pub type ShortGameName = String;

#[derive(Debug)]
pub struct REvilConfig {
    pub main: Main,
    pub games: HashMap<ShortGameName, GameConfig>,
}

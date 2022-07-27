#[cfg(test)]
use mockall::{automock};

use error_stack::{IntoReport, Result, ResultExt};
use std::fs::File;
use std::io::prelude::*;
use std::{
    fs::{self},
    path::Path,
};

use super::configStruct::{
    ConfigError::{self, *},
    REvilConfig,
};
use super::utils::{deserialize, serialize};

pub type ConfigResult<T> = Result<T, ConfigError>;

pub struct REvilConfigProvider {
    filename: String,
}

#[cfg_attr(test, automock)]
pub trait ConfigProvider {
    fn load_from_file(&self) -> ConfigResult<REvilConfig>;
    fn save_to_file(&self, config: &REvilConfig) -> ConfigResult<()>;
}

impl ConfigProvider for REvilConfigProvider {
    fn load_from_file(&self) -> ConfigResult<REvilConfig> {
        let content = fs::read_to_string(&self.filename)
            .report()
            .change_context(ConfigFile)
            .attach_printable_lazy(|| format!("Error reading {}", &self.filename))?;

        let (main, games) = deserialize(&content).change_context(Deserializer)?;
        Ok(REvilConfig { main, games })
    }

    fn save_to_file(&self, config: &REvilConfig) -> ConfigResult<()> {
        let content = serialize(config)?;
        let mut file = File::create(&self.filename)
            .report()
            .change_context(ConfigFile)?;
        file.write(content.as_bytes())
            .report()
            .change_context(ConfigFile)?;
        Ok(())
    }
}

impl REvilConfigProvider {
    pub fn new(path: impl AsRef<Path>) -> Self {
        REvilConfigProvider {
            filename: path.as_ref().to_str().unwrap().to_owned(),
        }
    }
}

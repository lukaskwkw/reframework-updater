use std::{collections::HashMap, error::Error, fmt};

use crate::{
    steam::SteamThings,
    tomlConf::{
        config::ConfigProvider,
        configStruct::{ErrorLevel, GameConfig, Main, REvilConfig, Runtime},
    },
    utils::{local_version::LocalFiles, progress_style},
    DynResult, GAMES,
};
use env_logger::Env;
use error_stack::{Report, Result, ResultExt};
use log::{debug, info, trace, warn};
use std::time::Duration;

use indicatif::{ProgressBar};

#[derive(Debug)]
pub struct REvilManagerError;
impl fmt::Display for REvilManagerError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("REvilManagerError error")
    }
}

impl Error for REvilManagerError {}

pub struct REvilManager {
    config: REvilConfig,
    config_provider: Box<dyn ConfigProvider>,
    steam_menago: Box<dyn SteamThings>,
    local_provider: Box<dyn LocalFiles>,
}

type ResultManagerErr<T> = Result<T, REvilManagerError>;

// pub trait Callback<'a>: Fn(&'a Report<REvilManagerError>, &'a mut REvilManager) {}

// impl<'a, T> Callback<'a> for T where T: Fn(&'a Report<REvilManagerError>, &'a mut REvilManager) {}

pub trait REvilThings {
    fn load_config(&mut self) -> Result<&mut Self, REvilManagerError>;
    fn load_config_cb(
        &mut self,
        cb: impl Fn(&Report<REvilManagerError>, &mut REvilManager),
    ) -> Result<&mut Self, REvilManagerError>;
    fn load_games_from_steam(&mut self) -> ResultManagerErr<&mut Self>;
    fn load_games_from_steam_cb(
        &mut self,
        cb: impl Fn(&Report<REvilManagerError>, &mut REvilManager),
    ) -> Result<&mut Self, REvilManagerError>;
    fn generate_main_defaults(&mut self) -> Result<&mut Self, REvilManagerError>;
    fn get_local_settings_per_game(&mut self) -> &mut Self;
    fn attach_logger(&mut self) -> ResultManagerErr<&mut Self>;
    fn save_config(&mut self) -> DynResult<&mut Self>;
    fn check_for_REFramework_update(&mut self) -> &mut Self;
    fn download_REFramework_update(&mut self) -> &mut Self;
    fn unzip_updates(&mut self) -> DynResult<&mut Self>;
    fn check_for_self_update(&mut self) -> DynResult<&mut Self>;
    fn self_update(&mut self) -> DynResult<&mut Self>;
    fn launch_game(&mut self) -> DynResult<&mut Self>;
}

impl REvilManager {
    pub fn new(
        config_provider: Box<dyn ConfigProvider>,
        local_provider: Box<dyn LocalFiles>,
        steam_menago: Box<dyn SteamThings>,
    ) -> Self {
        Self {
            config: REvilConfig {
                main: Main {
                    sources: None,
                    autoupdate: None,
                    steamExePath: None,
                    steamGamesIdToSearchFor: None,
                    errorLevel: Some(ErrorLevel::debug),
                },
                games: HashMap::new(),
            },
            config_provider,
            steam_menago,
            local_provider,
        }
    }
}

impl REvilThings for REvilManager {
    fn load_config(&mut self) -> Result<&mut Self, REvilManagerError> {
        let config = self
            .config_provider
            .load_from_file()
            .change_context(REvilManagerError)?;
        self.config = config;
        self.attach_logger();
        Ok(self)
    }

    fn load_config_cb(
        &mut self,
        cb: impl Fn(&Report<REvilManagerError>, &mut REvilManager),
    ) -> Result<&mut REvilManager, REvilManagerError> {
        let _: Result<&mut Self, REvilManagerError> = match self.load_config() {
            Ok(it) => Ok(it),
            Err(err) => {
                cb(&err, self);
                return Ok(self);
            }
        };
        Ok(self)
    }

    fn load_games_from_steam(&mut self) -> ResultManagerErr<&mut Self> {
        info!("Going to auto-detect games");
        let game_ids = GAMES.map(|(k, v)| k);
        let games_tuple_arr = self
            .steam_menago
            .get_games_locations(&game_ids.to_vec())
            .change_context(REvilManagerError)?;

        games_tuple_arr.iter().for_each(|(id, path)| {
            let (_, game_short_name) = GAMES.iter().find(|(game_id, _)| game_id == id).unwrap();

            #[cfg(debug_assertions)]
            info!("game detected name {}, \n path {:?}", game_short_name, path);

            self.config.games.insert(
                game_short_name.to_string(),
                GameConfig {
                    location: Some(path.display().to_string()),
                    steamId: Some(id.to_owned()),
                    versions: None,
                    nextgen: Some(false),
                    runtime: Some(Runtime::OpenVR),
                    runArgs: None,
                },
            );
        });

        Ok(self)
    }

    fn load_games_from_steam_cb(
        &mut self,
        cb: impl Fn(&Report<REvilManagerError>, &mut REvilManager),
    ) -> Result<&mut Self, REvilManagerError> {
        let _: Result<&mut Self, REvilManagerError> = match self
            .load_games_from_steam()
            .change_context(REvilManagerError)
        {
            Ok(it) => Ok(it),
            Err(err) => {
                cb(&err, self);
                return Ok(self);
            }
        };
        Ok(self)
    }

    fn generate_main_defaults(&mut self) -> Result<&mut Self, REvilManagerError> {
        todo!()
    }

    fn get_local_settings_per_game(&mut self) -> &mut Self {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(80).as_secs());
        pb.set_style(progress_style::getProgressStyle());
        for (short_name, config) in self.config.games.iter_mut() {
            let game_location = config.location.as_ref().unwrap();
            pb.set_message(format!("Loading config from {} ...", game_location));
            pb.tick();
            let local_config = self
                .local_provider
                .get_local_report_for_game(&game_location, short_name);
            config.runtime = local_config.runtime;
            if local_config.version.is_some() {
                config.versions = Some([local_config.version.unwrap()].to_vec());
            }
            config.nextgen = local_config.nextgen;
        }
        pb.finish_with_message("Done");

        trace!("Full config: \n {:#?}", self.config);
        self
    }

    fn attach_logger(&mut self) -> Result<&mut Self, REvilManagerError> {
        let env = Env::default().filter_or(
            "NONENENENE",
            self.config
                .main
                .errorLevel
                .as_ref()
                .ok_or(REvilManagerError)?
                .to_string(),
        );

        match env_logger::Builder::from_env(env).try_init() {
            Ok(it) => it,
            Err(err) => {
                debug!("Logger already initialized {}", err);
                return Ok(self);
            }
        };
        Ok(self)
    }

    fn save_config(&mut self) -> DynResult<&mut Self> {
        todo!()
    }

    fn check_for_REFramework_update(&mut self) -> &mut Self {
        todo!()
    }

    fn download_REFramework_update(&mut self) -> &mut Self {
        todo!()
    }

    fn unzip_updates(&mut self) -> DynResult<&mut Self> {
        todo!()
    }

    fn check_for_self_update(&mut self) -> DynResult<&mut Self> {
        todo!()
    }

    fn self_update(&mut self) -> DynResult<&mut Self> {
        todo!()
    }

    fn launch_game(&mut self) -> DynResult<&mut Self> {
        todo!()
    }
}

#![feature(explicit_generic_args_with_impl_trait)]

use std::{collections::HashMap, error::Error, fmt};

use crate::{
    refr_github::{ManageGithub, REFRGithub},
    steam::SteamThings,
    tomlConf::{
        config::ConfigProvider,
        configStruct::{ErrorLevel, GameConfig, Main, REvilConfig, Runtime},
    },
    utils::{
        init_logger::{self, init_logger},
        local_version::LocalFiles,
        progress_style,
        version_parser::isRepoVersionNewer,
    },
    DynResult, GAMES, NIGHTLY_RELEASE, REPO_OWNER, ARGS,
};
use env_logger::Env;
use error_stack::{Report, Result, ResultExt};
use log::{debug, info, log, trace, warn, Level};
use std::time::Duration;

use indicatif::ProgressBar;

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
    skip_next: bool,
    games_that_require_update: Vec<String>,
    config_provider: Box<dyn ConfigProvider>,
    steam_menago: Box<dyn SteamThings>,
    local_provider: Box<dyn LocalFiles>,
    github_release_manager: Option<Box<dyn ManageGithub>>,
    refr_ctor: fn(&str, &str, &str) -> REFRGithub,
}

type ResultManagerErr<T> = Result<T, REvilManagerError>;

// pub trait Callback<'a>: Fn(&'a Report<REvilManagerError>, &'a mut REvilManager) {}

// impl<'a, T> Callback<'a> for T where T: Fn(&'a Report<REvilManagerError>, &'a mut REvilManager) {}

pub trait REvilThings {
    fn load_config(&mut self) -> Result<&mut Self, REvilManagerError>;
    fn load_games_from_steam(&mut self) -> ResultManagerErr<&mut Self>;
    fn bind(
        &mut self,
        cb: impl Fn(&mut REvilManager) -> ResultManagerErr<&mut Self>,
        log_level: Level,
    ) -> &mut Self;
    fn or_log_err(
        &mut self,
        cb: impl Fn(&mut REvilManager) -> ResultManagerErr<&mut Self>,
        log_level: Level,
    ) -> &mut Self;
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
        github_constr: fn(&str, &str, &str) -> REFRGithub,
    ) -> Self {
        Self {
            config: REvilConfig {
                main: Main {
                    sources: None,
                    autoupdate: None,
                    steamExePath: None,
                    steamGamesIdToSearchFor: None,
                    errorLevel: None,
                    repo_owner: None,
                    chosen_source: None,
                },
                games: HashMap::new(),
            },
            skip_next: false,
            config_provider,
            steam_menago,
            local_provider,
            refr_ctor: github_constr,
            github_release_manager: None,
            games_that_require_update: [].to_vec(),
        }
    }
}

impl REvilThings for REvilManager {
    fn load_config(&mut self) -> Result<&mut Self, REvilManagerError> {
        let config = self
            .config_provider
            .load_from_file()
            .change_context(REvilManagerError)
            .or_else(|err| {
                self.attach_logger()?;
                self.config.main.errorLevel = Some(ErrorLevel::info);
                return Err(err);
            })?;
        self.config = config;
        self.attach_logger()?;
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
            // unwrap here is ok as we don't expect different game as GAMES where passed to get_games_locations earlier too
            let (_, game_short_name) = GAMES.iter().find(|(game_id, _)| game_id == id).unwrap();

            info!("game detected name {}, path {:?}", game_short_name, path);

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

    fn bind(
        &mut self,
        cb: impl Fn(&mut REvilManager) -> ResultManagerErr<&mut Self>,
        log_level: Level,
    ) -> &mut Self {
        if self.skip_next {
            return self;
        }
        match cb(self) {
            Ok(it) => return self,
            Err(err) => {
                self.skip_next = true;
                log!(log_level, "{}", err);
                debug!("Error {:?}", err);
                return self;
            }
        }
    }

    fn or_log_err(
        &mut self,
        cb: impl Fn(&mut REvilManager) -> ResultManagerErr<&mut Self>,
        log_level: Level,
    ) -> &mut Self {
        match cb(self) {
            Ok(it) => return self,
            Err(err) => {
                log!(log_level, "{}", err);
                debug!("Error {:?}", err);
                return self;
            }
        }
    }

    fn generate_main_defaults(&mut self) -> Result<&mut Self, REvilManagerError> {
        todo!()
    }

    fn get_local_settings_per_game(&mut self) -> &mut Self {
        info!("Checking local mod config per game");
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
            /* TODO this info doesnt show in console log check why or erase it
            also seems like because of progressbar some log have no chance to show up
            info!(
                "Local config for [{}], runtime [{:?}], nextgen [{:?}], version [{:?}]",
                short_name, config.runtime, local_config.nextgen, config.versions
            ); */
        }
        pb.finish_with_message("Done");

        trace!("Full config: \n {:#?}", self.config);
        self
    }

    fn attach_logger(&mut self) -> Result<&mut Self, REvilManagerError> {
        let level;
        unsafe {
            level = &ARGS.as_ref().unwrap().level;
        }
        init_logger(
            self.config
                .main
                .errorLevel
                .as_ref()
                .unwrap_or(level)
                .to_string()
                .as_ref(),
        );

        Ok(self)
    }

    fn save_config(&mut self) -> DynResult<&mut Self> {
        todo!()
    }

    fn check_for_REFramework_update(&mut self) -> &mut Self {
        let main = &self.config.main;
        let repo_owner: String = match &main.repo_owner {
            Some(it) => it.to_string(),
            None => REPO_OWNER.to_string(),
        };
        let source: String = match &main.chosen_source {
            Some(it) => it.to_string(),
            None => NIGHTLY_RELEASE.to_string(),
        };
        // TODO remove third param
        self.github_release_manager = Some(Box::new((self.refr_ctor)(&repo_owner, &source, "c")));

        info!("Checking if new release exists");
        let manager = self.github_release_manager.as_mut().unwrap();
        manager.get_reframework_latest_release().unwrap();
        let release = manager.getRelease();
        if release.is_some() {
            self.config.games.iter().for_each(|(short_name, game)| {
                if game.versions.is_some() {
                    let latest_local_version = game.versions.as_ref().unwrap().first().unwrap();
                    let latest_github_version = release.as_ref().unwrap().name.as_ref();
                    debug!(
                        "Local version [{}], repo version [{}] for {}",
                        latest_local_version, latest_github_version, short_name
                    );

                    let is_rnewer =
                        isRepoVersionNewer(&latest_local_version, &latest_github_version);
                    if is_rnewer.is_some() && is_rnewer.unwrap() {
                        self.games_that_require_update.push(short_name.to_string());
                    };
                };
            })
        };

        debug!(
            "games_that_require_update, {:?}",
            self.games_that_require_update
        );
        self
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

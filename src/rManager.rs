#![feature(explicit_generic_args_with_impl_trait)]

use std::{collections::HashMap, error::Error, fmt, cmp::Ordering, ops::RangeBounds, rc::Rc, any};

use crate::{
    create_TDB_string,
    refr_github::{ManageGithub, REFRGithub},
    steam::SteamThings,
    tomlConf::{
        config::ConfigProvider,
        configStruct::{ErrorLevel, GameConfig, Main, REvilConfig, Runtime, ShortGameName},
    },
    utils::{
        init_logger::{self, init_logger},
        local_version::LocalFiles,
        progress_style,
        version_parser::isRepoVersionNewer,
    },
    DynResult, ARGS, GAMES, GAMES_NEXTGEN_SUPPORT, NIGHTLY_RELEASE, REPO_OWNER, STANDARD_TYPE_QUALIFIER,
};
use dialoguer::{theme::ColorfulTheme, Select};
use env_logger::Env;
use error_stack::{Report, Result, ResultExt};
use log::{debug, info, log, trace, warn, Level};
use self_update::update::ReleaseAsset;
use std::time::Duration;

use indicatif::ProgressBar;

#[derive(Debug, Default)]
pub enum REvilManagerError {
    ReleaseIsEmpty,
    CheckingNewReleaseErr,
    ReleaseManagerIsNotInitialized,
    #[default]
    Other
}

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
    refr_ctor: fn(&str, &str) -> REFRGithub,
    selected_assets: Vec<ReleaseAsset>,
}

type ResultManagerErr<T> = Result<T, REvilManagerError>;
type FileName = String;

// pub trait Callback<'a>: Fn(&'a Report<REvilManagerError>, &'a mut REvilManager) {}

// impl<'a, T> Callback<'a> for T where T: Fn(&'a Report<REvilManagerError>, &'a mut REvilManager) {}

const SORT_DETERMINER: &str = "info";



pub trait REvilThings {
    fn load_config(&mut self) -> Result<&mut Self, REvilManagerError>;
    fn attach_logger(&mut self) -> ResultManagerErr<&mut Self>;
    fn load_games_from_steam(&mut self) -> ResultManagerErr<&mut Self>;
    fn generate_main_defaults(&mut self) -> Result<&mut Self, REvilManagerError>;
    fn get_local_settings_per_game(&mut self) -> &mut Self;
    fn check_for_REFramework_update(&mut self) -> ResultManagerErr<&mut Self>;
    fn ask_for_decision(&mut self) -> ResultManagerErr<&mut Self>;
    fn download_REFramework_update(&mut self) -> ResultManagerErr<&mut Self>;
    fn unzip_updates(&mut self) -> ResultManagerErr<&mut Self>;
    fn save_config(&mut self) -> DynResult<&mut Self>;
    fn check_for_self_update(&mut self) -> DynResult<&mut Self>;
    fn self_update(&mut self) -> DynResult<&mut Self>;
    fn launch_game(&mut self) -> DynResult<&mut Self>;
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
}

impl REvilManager {
    pub fn new(
        config_provider: Box<dyn ConfigProvider>,
        local_provider: Box<dyn LocalFiles>,
        steam_menago: Box<dyn SteamThings>,
        github_constr: fn(&str, &str) -> REFRGithub,
    ) -> Self {
        Self {
            config: REvilConfig::default(),
            skip_next: false,
            config_provider,
            steam_menago,
            local_provider,
            refr_ctor: github_constr,
            github_release_manager: None,
            games_that_require_update: [].to_vec(),
            selected_assets: Vec::new(),
            // selected_assets: Rc::new(Vec::new()),
        }
    }
    
    pub fn sort(a: &str, b: &str) -> Ordering {
        if a.contains(&SORT_DETERMINER) && !b.contains(&SORT_DETERMINER) {
            Ordering::Greater
        } else if !a.contains(&SORT_DETERMINER) && !b.contains(&SORT_DETERMINER)   {
            Ordering::Equal
        } else {
            Ordering::Less
        }
    }
    
    pub fn remove_second_runtime_file_if_applicable() {
        todo!();
    }
}

impl REvilThings for REvilManager {
    fn load_config(&mut self) -> Result<&mut Self, REvilManagerError> {
        let config = self
            .config_provider
            .load_from_file()
            .change_context(REvilManagerError::default())
            .or_else(|err| {
                self.attach_logger()?;
                self.config.main.errorLevel = Some(ErrorLevel::info);
                return Err(err);
            })?;
        self.config = config;
        self.attach_logger()?;
        Ok(self)
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

    fn load_games_from_steam(&mut self) -> ResultManagerErr<&mut Self> {
        info!("Going to auto-detect games");
        let game_ids = GAMES.map(|(k, v)| k);
        let games_tuple_arr = self
            .steam_menago
            .get_games_locations(&game_ids.to_vec())
            .change_context(REvilManagerError::default())?;

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

    fn check_for_REFramework_update(&mut self) -> ResultManagerErr<&mut Self> {
        let main = &self.config.main;
        let repo_owner: String = match &main.repo_owner {
            Some(it) => it.to_string(),
            None => REPO_OWNER.to_string(),
        };
        let source: String = match &main.chosen_source {
            Some(it) => it.to_string(),
            None => NIGHTLY_RELEASE.to_string(),
        };
        self.github_release_manager = Some(Box::new((self.refr_ctor)(&repo_owner, &source)));

        info!("Checking if new release exists");
        let manager = self.github_release_manager.as_mut().ok_or(Report::new(REvilManagerError::ReleaseManagerIsNotInitialized))?;
        manager.get_reframework_latest_release().or_else(|err| 
            Err(Report::new(REvilManagerError::CheckingNewReleaseErr)).attach_printable(format!("{:?}", err)))?;
            
        let release = manager.getRelease();
        self.config.games.iter().try_for_each(|(short_name, game)| -> ResultManagerErr<()> {
            if game.versions.is_some() {
                let latest_local_version = game.versions.as_ref().unwrap().first().unwrap();
                let latest_github_version = release.as_ref().ok_or(Report::new(REvilManagerError::ReleaseIsEmpty))?.name.as_ref();
                debug!(
                    "Local version [{}], repo version [{}] for {}",
                    latest_local_version, latest_github_version, short_name
                );

                let is_rnewer =
                    isRepoVersionNewer(&latest_local_version, &latest_github_version);
                if is_rnewer.is_some() && is_rnewer.unwrap() {
                    self.games_that_require_update.push(short_name.to_string());
                };
            } else {
                debug!("Version is None treating like needs to be added for {}.", short_name);
                self.games_that_require_update.push(short_name.to_string());
            };
            Ok(())
        })?;

        debug!(
            "games_that_require_update, {:?}",
            self.games_that_require_update
        );
        Ok(self)
    }

    // TODO consider testing scenario for games without NEXTGEN option like i.e. only ["MHRISE". "DCM5", "RE8"]
    fn ask_for_decision(&mut self) -> ResultManagerErr<&mut Self>  {
        // it determines wether you have game that supports different version i.e. RE2 support both nextgen and standard but if you have only game like 
        // MHRISE DMC5 then it should not change thus should not display specific message later
        let mut different_found = false;
        // it checks if any nextgen supported game doesn't have nextgen type set - treating like mod is not installed
        let mut any_none = false;
        let mut games: HashMap<String, (&ReleaseAsset, Option<bool>)> = HashMap::new(); 
        let rel_manager = self.github_release_manager.as_ref();
        let rel_manager = rel_manager.ok_or(Report::new(REvilManagerError::ReleaseManagerIsNotInitialized))?;
        let report = rel_manager.getAssetsReport();
        report.iter().for_each(|(short_name, assets)| {
            if self.games_that_require_update.contains(short_name) {
                assets.iter().for_each(|asset| {
                    let mut text;
                    let mut include_for_all_option = Some(true);
                    if GAMES_NEXTGEN_SUPPORT.contains(&&short_name[..]) {
                        different_found = true;
                        let nextgen = self.config.games.get(short_name).unwrap().nextgen;                    
                        let tdb = create_TDB_string(short_name);
                        if asset.name.contains(&tdb) {
                            text = format!("{} Standard version", short_name);
                            if nextgen.is_some() && nextgen.unwrap() {
                                include_for_all_option = Some(false);
                                text = format!("{}      {}(your current version of mod is nextgen -> it will switch to standard)", text, SORT_DETERMINER)
                            } else if nextgen.is_none() {
                                include_for_all_option = None;
                                any_none = true;
                            };
                        } else {
                            text = format!("{} Nextgen version", short_name);
                            if nextgen.is_some() && !nextgen.unwrap() {
                                include_for_all_option = Some(false);
                                text = format!("{}      {}(your current version of mod is standard -> it will switch to nextgen)", text, SORT_DETERMINER)
                            } else if nextgen.is_none() {
                                include_for_all_option = None;
                                any_none = true;
                            };
                        }
                    } else {
                        text = short_name.to_string();
                    }
                    games.insert(text, (asset, include_for_all_option));
                });
            }
        });
        let mut selections = ["Update all games".to_owned()].to_vec();
        if different_found && !any_none {
            selections[0] = format!("{} - (will choose base of your current local mod settings per game)", selections[0])
        } else if different_found && any_none {
            selections.push(format!("{} - prefer standard", selections[0]));
            selections[0] = format!("{} - prefer nextgen", selections[0]);
        }

        let mut texts: Vec<String> = games.keys().cloned().collect();
        texts.sort_by(|a,b|REvilManager::sort(&a,&b));
        selections.extend(texts);
        debug!("{:#?}", selections);

        let count = self.games_that_require_update.len();
        let mut additional_text = "";
        if different_found && any_none {
            additional_text = r"Also found that some of your games that
             can support both types Nextgen/Standard don't have mod installed.
             Chose which mod type use for them. For other games program will use correct version.";
        }
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("I found {} games that require update. Select which one you want to update or select all. {}", count, additional_text))
            .default(0)
            .items(&selections[..])
            .interact()
            .unwrap();
        
        debug!("selection {}, different_found {}, any_none {}", selection, different_found, any_none);
        
        if selection == 0 || (different_found && any_none && selection == 1)  {
            games.values().for_each(|(asset, include)| {
                if include.is_some() && include.unwrap() {
                    debug!("adding asset {}", asset.name);
                    return self.selected_assets.push(asset.clone().clone());
                }
                if include.is_none() && different_found && any_none && selection == 0 && !asset.name.contains(STANDARD_TYPE_QUALIFIER) {
                    debug!("adding nextgen asset for {}", asset.name);
                    return self.selected_assets.push(asset.clone().clone());
                }
                if include.is_none() && different_found && any_none && selection == 1 && asset.name.contains(STANDARD_TYPE_QUALIFIER) {
                    debug!("adding standard asset for {}", asset.name);
                    return self.selected_assets.push(asset.clone().clone());
                }
            });
                return Ok(self);
        }

        if let Some((asset, _)) = games.get(&selections[selection]) {
            self.selected_assets.push(asset.clone().clone());
        };
        Ok(self)
    }

    fn download_REFramework_update(&mut self) -> ResultManagerErr<&mut Self> {
        self.selected_assets.iter().try_for_each(|asset| -> ResultManagerErr<()> {
            self.github_release_manager
            .as_ref()
            .unwrap()
            .download_release_asset(asset)
            .or_else(|err| 
                Err(Report::new(REvilManagerError::default()))
                .attach_printable(format!("Error during downloading asset {} Error {:?}", asset.name, err)))?;
            Ok(())
        })?;
        Ok(self)
    }

    fn unzip_updates(&mut self) -> ResultManagerErr<&mut Self> {
        todo!()
    }

    fn save_config(&mut self) -> DynResult<&mut Self> {
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
}

// #[test]
// fn sort_test {
    // REvilManager::so
// }
use std::{collections::HashMap, error::Error, fmt, cmp::Ordering, path::Path, process, fs, ops::Index, ffi::OsStr};

use crate::{
    create_TDB_string,
    refr_github::{ManageGithub, REFRGithub},
    steam::SteamThings,
    tomlConf::{
        config::ConfigProvider,
        configStruct::{ErrorLevel, GameConfig, REvilConfig, Runtime, ShortGameName, SteamId},
    },
    utils::{
        init_logger::{init_logger},
        local_version::LocalFiles,
        progress_style,
        version_parser::{isRepoVersionNewer},
    },
    DynResult, ARGS, GAMES, GAMES_NEXTGEN_SUPPORT, NIGHTLY_RELEASE, REPO_OWNER, STANDARD_TYPE_QUALIFIER, unzip::{unzip}, reframework_github::release, MAX_ZIP_FILES_PER_GAME_CACHE,
};
use dialoguer::{theme::ColorfulTheme, Select};

use error_stack::{Report, Result, ResultExt, IntoReport, Context};
use log::{debug, info, log, trace, Level, warn};
use self_update::update::ReleaseAsset;
use std::time::Duration;

use indicatif::ProgressBar;

#[derive(Debug, Default)]
pub enum REvilManagerError {
    ReleaseIsEmpty,
    CheckingNewReleaseErr,
    GameNotFoundForGivenShortName(String),
    GameNotFoundForGivenSteamId(String),
    CannotDeductShortNameFromAssetName(String),
    RemoveFileFiled(String),
    RemoveZipAssetFromCacheErr(String),
    CacheNotFoundForGivenVersion(String),
    ReleaseManagerIsNotInitialized,
    GameLocationMissing,
    UnzipError,
    SaveConfigError,
    GameToLaunchIsNone,
    ReadDirError(String),
    LoadConfigError,
    #[default]
    Other
}

impl fmt::Display for REvilManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            REvilManagerError::ReleaseIsEmpty => write!(f, "ReleaseIsEmpty"),
            REvilManagerError::CheckingNewReleaseErr => write!(f, "CheckingNewReleaseErr"),
            REvilManagerError::GameNotFoundForGivenShortName(info) => write!(f, "GameNotFoundForGivenShortName {}", info),
            REvilManagerError::CannotDeductShortNameFromAssetName(info) => write!(f, "CannotDeductShortNameFromAssetName {}", info),
            REvilManagerError::RemoveZipAssetFromCacheErr(info) => write!(f, "RemoveZipAssetFromCacheErr {}", info),
            REvilManagerError::ReleaseManagerIsNotInitialized => write!(f, "ReleaseManagerIsNotInitialized"),
            REvilManagerError::GameLocationMissing => write!(f, "GameLocationMissing"),
            REvilManagerError::UnzipError => write!(f, "UnzipError"),
            REvilManagerError::SaveConfigError => write!(f, "SaveConfigError"),
            REvilManagerError::GameToLaunchIsNone => write!(f, "GameToLaunchIsNone"),
            REvilManagerError::ReadDirError(info) => write!(f, "ReadDirError {}", info),
            REvilManagerError::LoadConfigError => write!(f, "LoadConfigError"),
            REvilManagerError::Other => write!(f, "Other"),
            REvilManagerError::RemoveFileFiled(info) => write!(f, "RemoveFileFiled {}", info),
            REvilManagerError::GameNotFoundForGivenSteamId(info) => write!(f, "GameNotFoundForGivenSteamId {}", info),
            REvilManagerError::CacheNotFoundForGivenVersion(info) => write!(f, "CacheNotFoundForGivenVersion {}", info),
        }
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
    selected_game_to_launch: Option<SteamId>,
    pub config_loading_error_ocurred: bool,
}

type ResultManagerErr<T> = Result<T, REvilManagerError>;

const SORT_DETERMINER: &str = "info";

pub trait REvilThings {
    fn load_config(&mut self) -> ResultManagerErr<&mut Self>;
    fn attach_logger(&mut self) -> ResultManagerErr<&mut Self>;
    fn load_games_from_steam(&mut self) -> ResultManagerErr<&mut Self>;
    fn generate_main_defaults(&mut self) -> Result<&mut Self, REvilManagerError>;
    fn get_local_settings_per_game(&mut self) -> &mut Self;
    // fn get_local_settings_per_game_if_missing_conf_file(&mut self) -> &mut Self;
    fn check_for_REFramework_update(&mut self) -> ResultManagerErr<&mut Self>;
    fn ask_for_decision(&mut self) -> ResultManagerErr<&mut Self>;
    fn download_REFramework_update(&mut self) -> ResultManagerErr<&mut Self>;
    fn unzip_update<F: Fn(&OsStr) -> bool>(&self, game_short_name: &str, file_name: &str, version: Option<&str>, unzip_skip_fun: Option<F>) -> ResultManagerErr<&Self>
    where 
    F: Fn(&OsStr) -> bool;
    fn unzip_updates(&self) -> ResultManagerErr<&Self>;
    fn after_unzip_work(&mut self) -> Result<&mut Self, REvilManagerError>;
    fn save_config(&mut self) -> ResultManagerErr<&mut Self>;
    fn ask_for_game_decision_if_needed(&mut self) -> ResultManagerErr<&mut Self>;
    fn check_for_self_update(&mut self) -> DynResult<&mut Self>;
    fn self_update(&mut self) -> DynResult<&mut Self>;
    fn before_launch_procedure(&self, steam_id: &String) -> ResultManagerErr<()>;
    fn launch_game(&mut self) -> ResultManagerErr<&mut Self>;
    fn find_game_conf_by_steam_id(&self, steam_id: &String) -> ResultManagerErr<(&String, &GameConfig)>;
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
            config_loading_error_ocurred: false,
            selected_game_to_launch: None,
        }
    }

    pub fn unzip<F>(file: impl AsRef<Path>, destination: impl AsRef<Path>, runtime: &Option<Runtime>, skip_fun: Option<F>) -> ResultManagerErr<()> 
    where 
    F: Fn(&OsStr) -> bool {
        match runtime {
            Some(it) => {
                let should_skip = |file: &OsStr| file == OsStr::new(&it.as_opposite_local_dll());
                if skip_fun.is_none() {
                    unzip::unzip(file, destination, Some(should_skip)).change_context(REvilManagerError::UnzipError)?;
                } else {
                    unzip::unzip(file, destination, skip_fun).change_context(REvilManagerError::UnzipError)?;
                }
                return Ok(());
            },
            None => {
                let should_skip = |file: &OsStr| file == OsStr::new(&Runtime::OpenVR.as_opposite_local_dll());
                if skip_fun.is_none() {
                    unzip::unzip(file, destination, Some(should_skip)).change_context(REvilManagerError::UnzipError)?;
                } else {
                    unzip::unzip(file, destination, skip_fun).change_context(REvilManagerError::UnzipError)?;
                }
                return Ok(());
            },
        };
        return Ok(());
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
}

impl REvilThings for REvilManager {
    fn load_config(&mut self) -> ResultManagerErr<&mut Self> {
        let config = self
            .config_provider
            .load_from_file()
            .change_context(REvilManagerError::LoadConfigError)
            .or_else(|err| {
                self.config_loading_error_ocurred = true;
                self.attach_logger()?;
                self.config.main.errorLevel = Some(ErrorLevel::info);
                Err(err)
            })?;
        self.config = config;
        self.attach_logger()?;
        Ok(self)
    }

    fn attach_logger(&mut self) -> Result<&mut Self, REvilManagerError> {
        let mut level;
        unsafe {
            level = &ARGS.as_ref().unwrap().level;
        }
        if level == &ErrorLevel::none {
            level = self.config
            .main
            .errorLevel
            .as_ref()
            .unwrap_or(&ErrorLevel::info);
        }
        println!("Level {}", level);

        init_logger(level.to_string().as_ref());

        Ok(self)
    }

    fn load_games_from_steam(&mut self) -> ResultManagerErr<&mut Self> {
        info!("Going to auto-detect games");
        let game_ids = GAMES.map(|(k, _v)| k);
        let games_tuple_arr = self
            .steam_menago
            .get_games_locations(&game_ids.to_vec())
            .change_context(REvilManagerError::default())?;

        games_tuple_arr.iter().for_each(|(id, path)| {
            // unwrap call here is ok as we don't expect different game as GAMES where passed to get_games_locations earlier too
            let (_, game_short_name) = GAMES.iter().find(|(game_id, _)| game_id == id).unwrap();

            info!("game detected name {}, path {:?}", game_short_name, path);

            let game_config = GameConfig {
                location: Some(path.display().to_string()),
                steamId: Some(id.to_owned()),
                runtime: Some(Runtime::OpenVR),
                ..GameConfig::default()
            };

            self.config.games
                .entry(game_short_name.to_string())
                .and_modify(|game| {
                    GameConfig {
                        runtime: game.runtime.clone(),
                        nextgen: game.nextgen.clone(),
                        runArgs: game.runArgs.clone(),
                        ..game_config.clone()
                    };
                })
                .or_insert(game_config);
        });
        trace!("Steam configs after initialization {:#?}", self.config.games);
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
                .get_local_report_for_game(game_location, short_name);
            config.runtime = local_config.runtime;
            if local_config.version.is_some() {
                config.versions = Some([[local_config.version.unwrap()].to_vec()].to_vec());
            }
            config.nextgen = local_config.nextgen;
            /* TODO this info doesn't show in console log check why or erase it
            also seems like because of progress bar some log have no chance to show up
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
                    "Local version [{:?}], repo version [{}] for {}",
                    latest_local_version, latest_github_version, short_name
                );

                let is_rnewer =
                    isRepoVersionNewer(latest_local_version.first().unwrap(), latest_github_version);
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
        let mut games: HashMap<String, (&ReleaseAsset, Option<bool>, Option<SteamId>)> = HashMap::new(); 
        let rel_manager = self.github_release_manager.as_ref();
        let rel_manager = rel_manager.ok_or(Report::new(REvilManagerError::ReleaseManagerIsNotInitialized))?;
        let report = rel_manager.getAssetsReport();
        report.iter().for_each(|(short_name, assets)| {
            if self.games_that_require_update.contains(short_name) {
                assets.iter().for_each(|asset| {
                    let mut text;
                    let mut include_for_all_option = Some(true);
                    let game_config = self.config.games.get(short_name).unwrap();

                    if GAMES_NEXTGEN_SUPPORT.contains(&&short_name[..]) {
                        different_found = true;
                        let nextgen = game_config.nextgen;                    
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
                    games.insert(text, (asset, include_for_all_option, game_config.steamId.clone()));
                });
            }
        });
        let mut selections = vec![];
         if games.len() > 0 {
            selections.push("Update all games".to_string());
            if different_found && !any_none {
                selections[0] = format!("{} - (will choose base of your current local mod settings per game)", selections[0])
            } else if different_found && any_none {
                selections.push(format!("{} - prefer standard", selections[0]));
                selections[0] = format!("{} - prefer nextgen", selections[0]);
            }
        } else {
            info!("Not found any games to update");
            return Ok(self);
        }; 

        let mut texts: Vec<String> = games.keys().cloned().collect();
        texts.sort_by(|a,b|REvilManager::sort(a,b));
        selections.extend(texts);
        debug!("{:#?}", selections);

        let count = self.games_that_require_update.len();
        let mut additional_text = "";
        if different_found && any_none {
            additional_text = r"Also found that some of your games that
             can support both types Nextgen/Standard don't have mod installed.
             Chose which mod type use for them. For other games program will use correct version.";
        }
        selections.push("Skip".to_string());
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("I found {} games that require update. Select which one you want to update or select all. {}", count, additional_text))
            .default(0)
            .items(&selections[..])
            .interact()
            .unwrap();

        // important do not change order of below if call as later in iteration can provide out of index error
        if selection == (selections.len() -1) {
            info!("Chosen skip option.");
            return Ok(self);
        }
     
        debug!("selection {}, different_found {}, any_none {}", selection, different_found, any_none);
        
        if selection == 0 || (different_found && any_none && selection == 1)  {
            games.values().for_each(|(asset, include, _)| {
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
                    self.selected_assets.push(asset.clone().clone())
                }
            });
                return Ok(self);
        }

        if let Some((asset, _, game_id)) = games.get(&selections[selection]) {
            self.selected_assets.push(asset.clone().clone());
            self.selected_game_to_launch = game_id.clone();
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

    fn unzip_update<F>(&self, game_short_name: &str, file_name: &str, version: Option<&str>, unzip_skip_fun: Option<F>) -> ResultManagerErr<&Self> 
    where 
    F: Fn(&OsStr) -> bool
    {
        let game_config = self.config.games.get(game_short_name).ok_or(
            Report::new(REvilManagerError::GameNotFoundForGivenShortName(game_short_name.to_string())))?;
            let manager = self.github_release_manager.as_ref().ok_or(Report::new(REvilManagerError::ReleaseManagerIsNotInitialized))?;
            let path = manager.get_local_path_to_cache(version).or(Err(Report::new(REvilManagerError::ReleaseManagerIsNotInitialized)))?;
            let path = path.join(file_name);
            let location = game_config.location.as_ref().ok_or(Report::new(REvilManagerError::GameLocationMissing))?;
            REvilManager::unzip(path, location, &game_config.runtime, unzip_skip_fun)?;
        Ok(self)
    }

    fn unzip_updates(&self) -> ResultManagerErr<&Self> {
        let selected_assets =  &self.selected_assets;
        selected_assets.iter().try_for_each(|asset| -> ResultManagerErr<()> {

            let game_short_name = match asset.name.split_once(STANDARD_TYPE_QUALIFIER) {
                Some(tdb_asset) => Some(tdb_asset.0),
                None => match asset.name.split_once(".zip") {
                    Some(asset) => Some(asset.0),
                    None => None,
                },
            };

            if game_short_name.is_none() { return Err(Report::new(REvilManagerError::CannotDeductShortNameFromAssetName(asset.name.clone()))); };

            let game_short_name = game_short_name.unwrap();
            self.unzip_update::<fn(&OsStr) -> bool>(game_short_name, &asset.name, None, None)?;

            Ok(())
        })?;
        
        return Ok(self);
    }

    fn after_unzip_work(&mut self) -> Result<&mut Self, REvilManagerError> {
        let selected_assets =  &self.selected_assets;
        let manager = self.github_release_manager.as_ref().ok_or(Report::new(REvilManagerError::ReleaseManagerIsNotInitialized))?;
        let release = manager.getRelease();
        let version: &str = release.as_ref().ok_or(Report::new(REvilManagerError::ReleaseIsEmpty))?.name.as_ref();
        selected_assets.iter().try_for_each(|asset| -> ResultManagerErr<()> {
            info!("After unzip work - start");
            // for TDB assets STANDARD_TYPE_QUALIFIER is used and for rest games included nextgens ".zip"
            let game_short_name = match asset.name.split_once(STANDARD_TYPE_QUALIFIER) {
                Some(tdb_asset) => Some(tdb_asset.0),
                None => match asset.name.split_once(".zip") {
                    Some(asset) => Some(asset.0),
                    None => None,
                },
            };

            if game_short_name.is_none() { return Err(Report::new(REvilManagerError::CannotDeductShortNameFromAssetName(asset.name.clone()))); };
            let game_short_name = game_short_name.unwrap();         
            let game_config = self.config.games.get_mut(game_short_name).ok_or(
                Report::new(REvilManagerError::GameNotFoundForGivenShortName(game_short_name.to_string())))?;

            // add version from asset to array or create new array with the asset version
            if game_config.versions.is_some() {
                let versions = game_config.versions.as_mut().unwrap();
                versions.insert(0, [version.to_string(), asset.name.to_string()].to_vec());
            } else {
                game_config.versions = Some([[version.to_string(), asset.name.to_string()].to_vec()].to_vec());
            };

            // set NEXTGEN accordingly to an asset but only for the supported games
            if let Some(is_tdb) = does_asset_is_tdb(game_short_name, asset) {
                if is_tdb {
                    game_config.nextgen = Some(false);
                } else {
                    game_config.nextgen = Some(true);
                };
            };

            // remove second, not needed runtime file as for example when switching between different runtime versions 
            // second file may persists therefore blocking loading OpenXR runtime from loading
            remove_second_runtime_file(game_config)?;

            // it is ok to unwrap as in previous step we added array to that game config
            let versions = game_config.versions.as_ref().unwrap();
            if versions.len() > MAX_ZIP_FILES_PER_GAME_CACHE.into() {
                let last_ver = versions.last().unwrap();
                cleanup_cache(manager, last_ver, game_short_name)?;
                
                // after cleaning up cache remove last item from versions vector
                let mut versions = versions.clone();
                versions.pop();
                game_config.versions = Some(versions);
            }
            debug!("{:?}", game_config.versions);
            info!("After unzip work - done");
            Ok(())
        })?;
        
        return Ok(self);
    }

    fn save_config(&mut self) -> ResultManagerErr<&mut Self> {
        self
        .config_provider
        .save_to_file(&self.config)
        .change_context(REvilManagerError::SaveConfigError)?;
        Ok(self)
    }

    fn ask_for_game_decision_if_needed(&mut self) -> ResultManagerErr<&mut Self> {
        if self.selected_game_to_launch.is_some() {
            return Ok(self);
        }

        let mut selections_h_map: HashMap<String, &SteamId> = HashMap::new();

        &self.config.games.iter().for_each(|(short_name, game)| {
            if game.versions.is_none() {
                debug!("Game versions vector for {} not found", short_name);
                selections_h_map.insert(format!("Run {}", short_name), game.steamId.as_ref().unwrap());
                return;
            }
            let versions = game.versions.as_ref().unwrap();
            if versions.first().unwrap().len() > 1 && game.runtime.is_some() {
                selections_h_map.insert(format!("Run {} - Runtime <{:?}>", short_name, game.runtime.as_ref().unwrap()), game.steamId.as_ref().unwrap());
                selections_h_map.insert(format!("{}: Switch to <{:?}> runtime for {} and run game", SORT_DETERMINER, game.runtime.as_ref().unwrap().as_opposite(), short_name), game.steamId.as_ref().unwrap());
            } else {
                selections_h_map.insert(format!("Run {}", short_name), game.steamId.as_ref().unwrap());
            }
          });

        let mut selections: Vec<String> = selections_h_map.keys().cloned().collect();
        selections.sort_by(|a,b| REvilManager::sort(a, b));
        selections.push("Exit".to_string());
        let selection = Select::with_theme(&ColorfulTheme::default())
        
        .with_prompt(format!("Select game to run"))
        .default(0)
        .items(&selections[..])
        .interact()
        .unwrap();
        if selection == (selections.len() -1) {
            info!("Chosen exit option. Bye bye..");
            return Ok(self);
        }

        let selected_text = &selections[selection];
        let selected_steam_id = selections_h_map.get(&selections[selection]).unwrap().clone().to_string();

        if selected_text.contains(SORT_DETERMINER) {
            let (game_short_name, _) = self.find_game_conf_by_steam_id(&selected_steam_id)?;
            let game_short_name = game_short_name.clone();
            let game_config = self.config.games.get_mut(&game_short_name);
            let conf = game_config.unwrap();
            let runtime = conf.runtime.as_ref().unwrap();
            info!("Switching runtime {:?} to {:?} for {}", runtime, runtime.as_opposite(), game_short_name);
            conf.runtime = Some(runtime.as_opposite());
        }
        self.selected_game_to_launch = Some(selected_steam_id.clone().to_string());
        Ok(self)
    }

    fn check_for_self_update(&mut self) -> DynResult<&mut Self> {
        todo!()
    }

    fn self_update(&mut self) -> DynResult<&mut Self> {
        todo!()
    }

    fn launch_game(&mut self) -> ResultManagerErr<&mut Self>{
        if let Some(steam_id) = &self.selected_game_to_launch {
            self.before_launch_procedure(steam_id)?;
            
            info!("Launching the game");
            self.steam_menago.run_game_via_steam_manager(&steam_id).change_context(REvilManagerError::default())?
        } else {
            info!("Game to launch is none")
        };
        Ok(self)
    }

    fn before_launch_procedure(&self, steam_id: &String) -> ResultManagerErr<()> {
        let (game_short_name, game_config) = self.find_game_conf_by_steam_id(steam_id)?;
        if game_config.versions.is_none() {
            debug!("Version vector is empty for {}", game_short_name);
            return Ok(());
        }
        let version_vec = game_config.versions.as_ref().unwrap().first().unwrap();
        if version_vec.len() < 2 {
            debug!("Mod version has no cache file");
            return Ok(());
        }
        info!("Before launch procedure - start");
        let game_dir = game_config.location.as_ref().unwrap();
        let game_dir = Path::new(&game_dir);

        let runtime = game_config.runtime.as_ref().unwrap();
        if !game_dir.join(runtime.as_local_dll()).exists() {
            let should_skip_all_except = |file: &OsStr| file != OsStr::new(&runtime.as_local_dll());
            let ver = &version_vec[0];
            let file_name = &version_vec[1];

            self.unzip_update(game_short_name, &file_name, Some(&ver), Some(should_skip_all_except))?;
            info!("Unzipped only {} file", runtime.as_local_dll());
        }

        remove_second_runtime_file(game_config)?;

        info!("Before launch procedure - end");
        Ok(())
    }

    fn find_game_conf_by_steam_id(&self, steam_id: &String) -> ResultManagerErr<(&String, &GameConfig)> {
        let (game_short_name, game_config) = self.config.games.iter().find(|(_, conf)| {
            conf.steamId.as_ref().unwrap() == steam_id
        }).ok_or(
            Report::new(REvilManagerError::GameNotFoundForGivenSteamId(steam_id.to_string())))?;
        Ok((game_short_name, game_config))
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
            Ok(_it) => self,
            Err(err) => {
                self.skip_next = true;
                log!(log_level, "{}", err);
                debug!("Error {:?}", err);
                self
            }
        }
    }

    fn or_log_err(
        &mut self,
        cb: impl Fn(&mut REvilManager) -> ResultManagerErr<&mut Self>,
        log_level: Level,
    ) -> &mut Self {
        match cb(self) {
            Ok(_it) => self,
            Err(err) => {
                log!(log_level, "{}", err);
                debug!("Error {:?}", err);
                self
            }
        }
    }
}

fn remove_second_runtime_file(game_config: &GameConfig) -> ResultManagerErr<()> {
    let game_folder = Path::new(game_config.location.as_ref().unwrap());
    let open_runtime_path = game_folder.join(game_config.runtime.as_ref().unwrap().as_opposite_local_dll());
    Ok(if Path::new(&open_runtime_path).exists() {
        fs::remove_file(&open_runtime_path).report().change_context(REvilManagerError::RemoveFileFiled(open_runtime_path.display().to_string()))?;
        debug!("Second runtime file removed {}", open_runtime_path.display());
    } else {
        debug!("Second runtime file doesn't exist {}", open_runtime_path.display());
    })
}


fn cleanup_cache(manager: &Box<dyn ManageGithub<REFRGithub>>, last_ver: &Vec<String>, game_short_name: &str) -> ResultManagerErr<()> {
    if last_ver.len() < 2 {
        debug!("A Game {} Cache warn: {:?}", game_short_name, REvilManagerError::CacheNotFoundForGivenVersion(last_ver[0].to_string()).to_string());
        return Ok(());
    }
    let last_ver_nb = &last_ver[0];
    let cache_dir = manager.get_local_path_to_cache(Some(&last_ver_nb)).or(Err(Report::new(REvilManagerError::ReleaseManagerIsNotInitialized)))?;
    Ok(if cache_dir.exists() {
    
    let file_to_remove = cache_dir.join(last_ver[1].to_string());
    if Path::new(&file_to_remove).exists()  {
        fs::remove_file(&file_to_remove).report().change_context(REvilManagerError::RemoveZipAssetFromCacheErr(file_to_remove.display().to_string()))?;
    }
    match fs::remove_dir(&cache_dir) {
            Ok(()) => debug!("Directory: {} Removed",  cache_dir.display().to_string()),
            Err(err) => debug!("Can not Remove directory: {} Err {}",  cache_dir.display().to_string(),err),
        };
     })
}

// check if asset is TDB or not if it doesn't support nextgen version then None is returned
fn does_asset_is_tdb(game_short_name: &str, asset: &ReleaseAsset) -> Option<bool> {
    if GAMES_NEXTGEN_SUPPORT.contains(&game_short_name) {
        if asset.name.contains(STANDARD_TYPE_QUALIFIER) { 
            return Some(true);
             } 
            else {
            return Some(false);
        }
    }
    None
}

// #[test]
// fn sort_test {
    // REvilManager::so
// }
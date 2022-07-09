use std::{collections::HashMap, error::Error, fmt, cmp::Ordering, path::Path, process, fs};

use crate::{
    create_TDB_string,
    refr_github::{ManageGithub, REFRGithub},
    steam::SteamThings,
    tomlConf::{
        config::ConfigProvider,
        configStruct::{ErrorLevel, GameConfig, REvilConfig, Runtime},
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
    CannotDeductShortNameFromAssetName(String),
    RemoveZipAssetFromCacheErr(String),
    ReleaseManagerIsNotInitialized,
    GameLocationMissing,
    UnzipError,
    SaveConfigError,
    ReadDirError(String),
    LoadConfigError,
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
    pub config_loading_error_ocurred: bool,
}

type ResultManagerErr<T> = Result<T, REvilManagerError>;
type FileName = String;

// pub trait Callback<'a>: Fn(&'a Report<REvilManagerError>, &'a mut REvilManager) {}

// impl<'a, T> Callback<'a> for T where T: Fn(&'a Report<REvilManagerError>, &'a mut REvilManager) {}

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
    fn unzip_update(&self, game_short_name: &str, file_name: &str) -> ResultManagerErr<&Self>;
    fn unzip_updates(&self) -> ResultManagerErr<&Self>;
    fn after_unzip_work(&mut self) -> Result<&mut Self, REvilManagerError>;
    fn save_config(&mut self) -> ResultManagerErr<&mut Self>;
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
            config_loading_error_ocurred: false,
        }
    }

    pub fn unzip(file: impl AsRef<Path>, destination: impl AsRef<Path>, runtime: &Option<Runtime>) -> ResultManagerErr<()> {
        match runtime {
        Some(it) => {
            unzip::unzip([it.as_opposite_local_dll().as_ref()].to_vec(), file, destination, false).change_context(REvilManagerError::UnzipError)?
        },
        None => {
            unzip::unzip([Runtime::OpenVR.as_opposite_local_dll().as_ref()].to_vec(), file, destination, false).change_context(REvilManagerError::UnzipError)? 

        },
        };
        Ok(())
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
                    isRepoVersionNewer(latest_local_version, latest_github_version);
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
        let mut selections = vec![];
         if games.len() > 0 {
            selections.push("Update all games".to_string());
            if different_found && !any_none {
                selections[0] = format!("{} - (will choose base of your current local mod settings per game)", selections[0])
            } else if different_found && any_none {
                selections.push(format!("{} - prefer standard", selections[0]));
                selections[0] = format!("{} - prefer nextgen", selections[0]);
            }
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
        selections.push("Exit".to_string());
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("I found {} games that require update. Select which one you want to update or select all. {}", count, additional_text))
            .default(0)
            .items(&selections[..])
            .interact()
            .unwrap();

        // important do not change order of below if call as later in iteration can provide out of index error
        if selection == (selections.len() -1) {
            info!("Chosen exit option. Bye bye..");
            return Ok(self);
        }
        
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
                    self.selected_assets.push(asset.clone().clone())
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

    fn unzip_update(&self, game_short_name: &str, file_name: &str) -> ResultManagerErr<&Self> {
        let game_config = self.config.games.get(game_short_name).ok_or(
            Report::new(REvilManagerError::GameNotFoundForGivenShortName(game_short_name.to_string())))?;
            let manager = self.github_release_manager.as_ref().ok_or(Report::new(REvilManagerError::ReleaseManagerIsNotInitialized))?;
            let path = manager.get_local_path_to_cache(None).or(Err(Report::new(REvilManagerError::ReleaseManagerIsNotInitialized)))?;
            let path = path.join(file_name);
            let location = game_config.location.as_ref().ok_or(Report::new(REvilManagerError::GameLocationMissing))?;
            REvilManager::unzip(path, location, &game_config.runtime)?;
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
            self.unzip_update(game_short_name, &asset.name)?;

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
            info!("After unzip work for - start");
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


            if game_config.versions.is_some() {
                let versions = game_config.versions.as_mut().unwrap();
                versions.insert(0, version.to_string());

            } else {
                game_config.versions = Some([version.to_string()].to_vec());
            };

            // it is ok to unwrap as in previous step we added array to that game config
            let versions = game_config.versions.as_ref().unwrap();
            if versions.len() > MAX_ZIP_FILES_PER_GAME_CACHE.into() {
                let last_ver = versions.last().unwrap();
                // if local version is just a hash (not contains '.') then probably there is no backup folder for it too so don't try to delete this file
                // TODO but this can change if considered adding support for zipping first discovered mod version and preserving it
                if last_ver.contains('.') {
                   let cache_dir = manager.get_local_path_to_cache(Some(last_ver)).or(Err(Report::new(REvilManagerError::ReleaseManagerIsNotInitialized)))?;

                    let mut second_version_of_archive : Option<String> = None;
                    if GAMES_NEXTGEN_SUPPORT.contains(&game_short_name) {
                        if asset.name.contains(STANDARD_TYPE_QUALIFIER) { second_version_of_archive = Some(format!("{}.zip", game_short_name)); } else {
                            second_version_of_archive = Some(create_TDB_string(game_short_name));
                        }
                    }

                    for entry in fs::read_dir(&cache_dir).report().change_context(REvilManagerError::ReadDirError(cache_dir.display().to_string()))? {
                        let entry = entry.report().change_context(REvilManagerError::ReadDirError(format!("Entry err for {}", cache_dir.display().to_string())))?;
                        let path = entry.path();
                        if path.is_file() {
                            let file_name = path.file_name().unwrap();
                            let file_name = match file_name.to_str() {
                                Some(it) => it,
                                None => {
                                    debug!("File_name to_str failed: {:?}", file_name);
                                    "anything.txt"
                                },
                            };

                            if file_name == &asset.name {
                                fs::remove_file(&path).report().change_context(REvilManagerError::RemoveZipAssetFromCacheErr(path.display().to_string()))?;
                                debug!("File: {} Removed",  path.display().to_string());
                            }
                            if second_version_of_archive.is_some() && file_name.contains(second_version_of_archive.as_ref().unwrap()) {
                                fs::remove_file(&path).report().change_context(REvilManagerError::RemoveZipAssetFromCacheErr(path.display().to_string()))?;
                                debug!("File: {} Removed",  path.display().to_string());
                            }
                        }
                    }

                    match fs::remove_dir(&cache_dir) {
                        Ok(()) => debug!("Directory: {} Removed",  cache_dir.display().to_string()),
                        Err(err) => debug!("Can not Remove directory: {} Err {}",  cache_dir.display().to_string(),err),
                    };
               };
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

// #[test]
// fn sort_test {
    // REvilManager::so
// }
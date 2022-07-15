use std::{
    cmp::Ordering,
    env,
    ffi::OsStr,
    fs,
    path::Path,
    process::{self, Command},
};

use crate::{
    dialogs::dialogs::{Ask, SwitchActionReport},
    rManager::cleanup_cache::cleanup_cache,
    rManager::rManager_header::{
        REvilManager, REvilManagerError, REvilManagerState, REvilThings, ResultManagerErr,
        SORT_DETERMINER,
    },
    refr_github::REFRGithub,
    steam::SteamThings,
    tomlConf::{
        config::ConfigProvider,
        configStruct::{ErrorLevel, GameConfig, REvilConfig, Runtime},
    },
    unzip::unzip,
    utils::{
        init_logger::init_logger, local_version::LocalFiles, mslink::create_ms_lnk, progress_style,
        version_parser::isRepoVersionNewer, find_game_conf_by_steam_id::find_game_conf_by_steam_id, is_asset_tdb::is_asset_tdb,
    },
    DynResult, ARGS, GAMES, MAX_ZIP_FILES_PER_GAME_CACHE, NIGHTLY_RELEASE,
    REPO_OWNER, STANDARD_TYPE_QUALIFIER,
};

use error_stack::{IntoReport, Report, Result, ResultExt};
use log::{debug, info, log, trace, warn, Level};
use std::time::Duration;

use indicatif::ProgressBar;

pub static SWITCH_IDENTIFIER: &str = "switch";

impl REvilManager {
    pub fn new(
        config_provider: Box<dyn ConfigProvider>,
        local_provider: Box<dyn LocalFiles>,
        steam_menago: Box<dyn SteamThings>,
        dialogs: Box<dyn Ask>,
        github_constr: fn(&str, &str) -> REFRGithub,
    ) -> Self {
        Self {
            config: REvilConfig::default(),
            config_provider,
            steam_menago,
            local_provider,
            refr_ctor: github_constr,
            github_release_manager: None,
            state: REvilManagerState::default(),
            dialogs,
        }
    }

    pub fn unzip<F>(
        file: impl AsRef<Path>,
        destination: impl AsRef<Path>,
        runtime: &Option<Runtime>,
        skip_fun: Option<F>,
    ) -> ResultManagerErr<()>
    where
        F: Fn(&OsStr) -> bool,
    {
        if skip_fun.is_some() {
            unzip::unzip(&file, &destination, skip_fun)
                .change_context(REvilManagerError::UnzipError)?;
            return Ok(());
        };
        let closure = runtime
            .as_ref()
            .map(|runtime| -> Box<dyn Fn(&OsStr) -> bool> {
                let should_skip = |f: &OsStr| f == OsStr::new(&runtime.as_opposite_local_dll());
                return Box::new(should_skip);
            })
            .unwrap_or_else(|| {
                let should_skip =
                    |f: &OsStr| f == OsStr::new(&Runtime::OpenVR.as_opposite_local_dll());
                return Box::new(should_skip);
            });

        unzip::unzip(file, destination, Some(closure))
            .change_context(REvilManagerError::UnzipError)?;
        Ok(())
    }

    pub fn sort(a: &str, b: &str) -> Ordering {
        if a.contains(&SORT_DETERMINER) && !b.contains(&SORT_DETERMINER) {
            Ordering::Greater
        } else if !a.contains(&SORT_DETERMINER) && !b.contains(&SORT_DETERMINER) {
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
                self.state.config_loading_error_ocurred = true;
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
            level = self
                .config
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

            let len_before = self.config.games.len();
            self.config
                .games
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
            let len_after = self.config.games.len();
            if len_after > len_before {
                self.state.new_steam_game_found = true;
            }
        });
        trace!(
            "Steam configs after initialization {:#?}",
            self.config.games
        );
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

            // check fo config.versions  is called because maybe found new steam game and we don't want to
            // replace versions information for other games
            if local_config.version.is_some() && config.versions.is_none() {
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

    fn generate_ms_links(&mut self) -> ResultManagerErr<&mut Self> {
        match env::current_exe() {
            Ok(current_exe_path) => {
                let ms_links_folder = Path::new("REFR_links");
                fs::create_dir_all(&ms_links_folder).map_err(|err| {
                    Report::new(REvilManagerError::FailedToCreateMsLink(format!(
                        "Error during create_dir_all path {} Err {}",
                        ms_links_folder.display(),
                        err
                    )))
                })?;

                self.config.games.iter().try_for_each(
                    |(short_name, _)| -> ResultManagerErr<()> {
                        let ms_link_name = format!("REFR_{}.lnk", short_name);
                        let ms_link_path = ms_links_folder.join(Path::new(&ms_link_name));
                        if ms_link_path.exists() {
                            debug!(
                                "Ms link already exists for {} Path {}",
                                short_name,
                                ms_link_path.display()
                            );
                            return Ok(());
                        }

                        let arguments = format!("--one {}", short_name);
                        match create_ms_lnk(
                            &ms_link_path,
                            &current_exe_path,
                            Some(arguments.clone()),
                        )
                        .or_else(|err| {
                            Err(Report::new(REvilManagerError::FailedToCreateMsLink(
                                format!(
                                    "Failed for {} Ms Link path {} Current exe path {} args {}",
                                    short_name,
                                    ms_link_path.display(),
                                    current_exe_path.display(),
                                    arguments
                                ),
                            )))
                            .attach_printable(format!("{:?}", err))
                        }) {
                            Ok(_) => info!("Ms link created for {}", short_name),
                            Err(err) => {
                                warn!("{}", err);
                                debug!("{:?}", err);
                            }
                        };
                        return Ok(());
                    },
                )?;

                return Ok(self);
            }
            Err(err) => {
                return Err(Report::new(REvilManagerError::FailedToCreateMsLink(
                    format!("current_exe error: {}", err),
                )));
            }
        };
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
        let manager = self.github_release_manager.as_mut().ok_or(Report::new(
            REvilManagerError::ReleaseManagerIsNotInitialized,
        ))?;
        manager.get_reframework_latest_release().or_else(|err| {
            Err(Report::new(REvilManagerError::CheckingNewReleaseErr))
                .attach_printable(format!("{:?}", err))
        })?;

        // requires github_release_manager to be initialized
        self.set_games_that_require_update()?;

        debug!(
            "games_that_require_update, {:?}",
            self.state.games_that_require_update
        );
        Ok(self)
    }

    fn pick_one_game_from_report(&mut self) -> ResultManagerErr<&mut Self> {
        let game_short_name;
        let should_run_after;
        unsafe {
            game_short_name = &ARGS.as_ref().unwrap().one;
            should_run_after = &ARGS.as_ref().unwrap().run;
        }
        debug!("Args one {}, run {:?}", game_short_name, should_run_after);
        let game_config = self.config.games.get(game_short_name).unwrap();
        let steam_id = game_config.steamId.as_ref().unwrap();
        if should_run_after.to_logical() {
            self.state.selected_game_to_launch = Some(steam_id.to_string());
        };
        if !self
            .state
            .games_that_require_update
            .contains(&game_short_name.to_string())
        {
            info!("Update not required");
            return Ok(self);
        }
        let rel_manager = self.github_release_manager.as_ref();
        let rel_manager = rel_manager.ok_or(Report::new(
            REvilManagerError::ReleaseManagerIsNotInitialized,
        ))?;
        let report = rel_manager.getAssetsReport();
        let nextgen = game_config.nextgen.unwrap();
        report
            .iter()
            .find(|(short_name, _)| *short_name == game_short_name)
            .and_then(|(short_name, assets)| {
                assets.iter().for_each(|asset| {
                    is_asset_tdb(short_name, asset)
                        .map(|does| {
                            if (does && !nextgen) || (!does && nextgen) {
                                debug!("Added asset to download: {}", asset.name);
                                self.state.selected_assets.push(asset.clone())
                            };
                        })
                        .unwrap_or_else(|| {
                            debug!("un_or_else Added asset to download: {}", asset.name);
                            self.state.selected_assets.push(asset.clone())
                        });
                });
                return Some(());
            })
            .unwrap_or_else(|| debug!("Report doesn't contain {} game", game_short_name));

        Ok(self)
    }

    // TODO consider testing scenario for games without NEXTGEN option like i.e. only ["MHRISE". "DCM5", "RE8"]
    fn ask_for_decision(&mut self) -> ResultManagerErr<&mut Self> {
        let report = self
            .github_release_manager
            .as_ref()
            .map(|men| men.getAssetsReport())
            .ok_or(Report::new(
                REvilManagerError::ReleaseManagerIsNotInitialized,
            ))?;
        self.dialogs
            .ask_for_decision(&mut self.config, &mut self.state, report)
            .change_context(REvilManagerError::Other)?;
        Ok(self)
    }

    fn download_REFramework_update(&mut self) -> ResultManagerErr<&mut Self> {
        self.state
            .selected_assets
            .iter()
            .try_for_each(|asset| -> ResultManagerErr<()> {
                self.github_release_manager
                    .as_ref()
                    .unwrap()
                    .download_release_asset(asset)
                    .or_else(|err| {
                        Err(Report::new(REvilManagerError::default())).attach_printable(format!(
                            "Error during downloading asset {} Error {:?}",
                            asset.name, err
                        ))
                    })?;
                Ok(())
            })?;
        Ok(self)
    }

    fn unzip_update<F>(
        &self,
        game_short_name: &str,
        file_name: &str,
        version: Option<&str>,
        unzip_skip_fun: Option<F>,
    ) -> ResultManagerErr<&Self>
    where
        F: Fn(&OsStr) -> bool,
    {
        let game_config = self.config.games.get(game_short_name).ok_or(Report::new(
            REvilManagerError::GameNotFoundForGivenShortName(game_short_name.to_string()),
        ))?;
        let manager = self.github_release_manager.as_ref().ok_or(Report::new(
            REvilManagerError::ReleaseManagerIsNotInitialized,
        ))?;
        let path = manager
            .get_local_path_to_cache(version)
            .map(|path| path.join(file_name))
            .or(Err(Report::new(
                REvilManagerError::ReleaseManagerIsNotInitialized,
            )))?;
        let location = game_config
            .location
            .as_ref()
            .ok_or(Report::new(REvilManagerError::GameLocationMissing))?;
        REvilManager::unzip(path, location, &game_config.runtime, unzip_skip_fun)?;
        Ok(self)
    }

    fn unzip_updates(&self) -> ResultManagerErr<&Self> {
        let selected_assets = &self.state.selected_assets;
        selected_assets
            .iter()
            .try_for_each(|asset| -> ResultManagerErr<()> {
                let game_short_name = match asset.name.split_once(STANDARD_TYPE_QUALIFIER) {
                    Some(tdb_asset) => Some(tdb_asset.0),
                    None => match asset.name.split_once(".zip") {
                        Some(asset) => Some(asset.0),
                        None => None,
                    },
                };

                if game_short_name.is_none() {
                    return Err(Report::new(
                        REvilManagerError::CannotDeductShortNameFromAssetName(asset.name.clone()),
                    ));
                };

                let game_short_name = game_short_name.unwrap();
                self.unzip_update::<fn(&OsStr) -> bool>(game_short_name, &asset.name, None, None)?;

                Ok(())
            })?;

        return Ok(self);
    }

    fn after_unzip_work(&mut self) -> Result<&mut Self, REvilManagerError> {
        let selected_assets = &self.state.selected_assets;
        let manager = self.github_release_manager.as_ref().ok_or(Report::new(
            REvilManagerError::ReleaseManagerIsNotInitialized,
        ))?;
        let release = manager.getRelease();
        let version: &str = release
            .as_ref()
            .ok_or(Report::new(REvilManagerError::ReleaseIsEmpty))?
            .name
            .as_ref();
        selected_assets
            .iter()
            .try_for_each(|asset| -> ResultManagerErr<()> {
                info!("After unzip work - start");
                // for TDB assets STANDARD_TYPE_QUALIFIER is used and for rest games included nextgens ".zip"
                let game_short_name = asset
                    .name
                    .split_once(STANDARD_TYPE_QUALIFIER)
                    .and_then(|(short_name, _)| Some(short_name))
                    .or_else(|| {
                        asset
                            .name
                            .split_once(".zip")
                            .and_then(|(short_name, _)| Some(short_name))
                    })
                    .ok_or(Report::new(
                        REvilManagerError::CannotDeductShortNameFromAssetName(asset.name.clone()),
                    ))?;

                // remove game from req_update_games vec as it is already updated!
                let req_update_games: &mut Vec<String> =
                    self.state.games_that_require_update.as_mut();

                req_update_games
                    .iter()
                    .position(|sn| sn == game_short_name)
                    .map(|pos| req_update_games.remove(pos))
                    .unwrap_or_default();

                let game_short_name = game_short_name;
                let game_config = self
                    .config
                    .games
                    .get_mut(game_short_name)
                    .ok_or(Report::new(
                        REvilManagerError::GameNotFoundForGivenShortName(
                            game_short_name.to_string(),
                        ),
                    ))?;

                // add version from asset to array or create new array with the asset version
                game_config
                    .versions
                    .as_mut()
                    .map(|versions| {
                        let first_set = versions.first().unwrap();
                        if first_set[0] == SWITCH_IDENTIFIER {
                            if first_set.len() > 1 {
                                let vecc = [
                                    version.to_string(),
                                    asset.name.to_string(),
                                    first_set[1].to_string(),
                                ]
                                .to_vec();
                                versions.remove(0);
                                debug!("switch more than 1 asset {}", asset.name);
                                versions.insert(0, vecc);
                            } else {
                                versions.remove(0);
                                versions.insert(
                                    0,
                                    [version.to_string(), asset.name.to_string()].to_vec(),
                                );
                                debug!("switch less than 1 asset {}", asset.name);
                            }
                        } else {
                            debug!("no switch asset {}", asset.name);
                            versions
                                .insert(0, [version.to_string(), asset.name.to_string()].to_vec())
                        }
                    })
                    .unwrap_or_else(|| {
                        game_config.versions =
                            Some([[version.to_string(), asset.name.to_string()].to_vec()].to_vec())
                    });

                // set NEXTGEN accordingly to an asset but only for the supported games
                is_asset_tdb(game_short_name, asset)
                    .map(|does| game_config.nextgen = Some(!does))
                    .unwrap_or_default();

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
        info!("Saving tool config");
        self.config_provider
            .save_to_file(&self.config)
            .change_context(REvilManagerError::SaveConfigError)?;
        Ok(self)
    }

    fn ask_for_game_decision_if_needed(&mut self) -> ResultManagerErr<&mut Self> {
        self.dialogs
            .ask_for_game_decision_if_needed(&mut self.config, &mut self.state)
            .change_context(REvilManagerError::Other)?;
        Ok(self)
    }

    fn ask_for_switch_type_decision(&mut self) -> ResultManagerErr<&mut Self> {
        let what_next = self
            .dialogs
            .ask_for_switch_type_decision(&mut self.config, &mut self.state)
            .change_context(REvilManagerError::Other)?;
        use SwitchActionReport::*;
        match what_next {
            UnzipSaveAndExit(short_name, second_asset_name) => {
                self.unzip_update::<fn(&OsStr) -> bool>(
                    &short_name,
                    &second_asset_name,
                    None,
                    None,
                )?;
                self.save_config()?;
                process::exit(0);
            }
            SaveAndRunThenExit(short_name, path) => {
                self.save_config()?;
                if cfg!(target_os = "windows") {
                    Command::new(path)
                        .args(["-r", "no", "--one", &short_name])
                        .spawn()
                        .expect("failed to execute process");
                    process::exit(0);
                };
            }
            Early => {
                return Ok(self);
            }
        }
        Ok(self)
    }

    fn check_for_self_update(&mut self) -> DynResult<&mut Self> {
        todo!()
    }

    fn self_update(&mut self) -> DynResult<&mut Self> {
        todo!()
    }

    fn before_launch_procedure(&self, steam_id: &String) -> ResultManagerErr<()> {
        let (game_short_name, game_config) = find_game_conf_by_steam_id(&self.config, steam_id)?;
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
            // TODO consider extracting from suitable mod type i.e. for standard use TDB asset, for nextgen i.e. RE7.zip
            let file_name = &version_vec[1];

            self.unzip_update(
                game_short_name,
                &file_name,
                Some(&ver),
                Some(should_skip_all_except),
            )?;
            info!("Unzipped only {} file", runtime.as_local_dll());
        }

        remove_second_runtime_file(game_config)?;

        info!("Before launch procedure - end");
        Ok(())
    }

    fn launch_game(&mut self) -> ResultManagerErr<&mut Self> {
        if let Some(steam_id) = &self.state.selected_game_to_launch {
            self.before_launch_procedure(steam_id)?;

            info!("Launching the game");
            self.steam_menago
                .run_game_via_steam_manager(&steam_id)
                .change_context(REvilManagerError::default())?
        } else {
            info!("Game to launch is none")
        };
        Ok(self)
    }

    fn bind(
        &mut self,
        cb: impl Fn(&mut REvilManager) -> ResultManagerErr<&mut Self>,
        log_level: Level,
    ) -> &mut Self {
        if self.state.skip_next {
            return self;
        }
        match cb(self) {
            Ok(_it) => self,
            Err(err) => {
                self.state.skip_next = true;
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

impl REvilManager {
    fn set_games_that_require_update(&mut self) -> ResultManagerErr<()> {
        let manager = self.github_release_manager.as_mut().ok_or(Report::new(
            REvilManagerError::ReleaseManagerIsNotInitialized,
        ))?;
        let release = manager.getRelease();
        self.config
            .games
            .iter()
            .try_for_each(|(short_name, game)| -> ResultManagerErr<()> {
                if game.versions.is_some() {
                    let latest_local_version = game.versions.as_ref().unwrap().first().unwrap();
                    let latest_github_version = release
                        .as_ref()
                        .ok_or(Report::new(REvilManagerError::ReleaseIsEmpty))?
                        .name
                        .as_ref();
                    debug!(
                        "Local version [{:?}], repo version [{}] for {}",
                        latest_local_version, latest_github_version, short_name
                    );

                    let is_rnewer = isRepoVersionNewer(
                        latest_local_version.first().unwrap(),
                        latest_github_version,
                    );
                    is_rnewer.map(|is| {
                        is.then(|| {
                            self.state
                                .games_that_require_update
                                .push(short_name.to_string())
                        })
                        .unwrap_or(())
                    });
                } else {
                    debug!(
                        "Version is None treating like needs to be added for {}.",
                        short_name
                    );
                    self.state
                        .games_that_require_update
                        .push(short_name.to_string());
                };
                Ok(())
            })?;
        Ok(())
    }
}

fn remove_second_runtime_file(game_config: &GameConfig) -> ResultManagerErr<()> {
    let game_folder = Path::new(game_config.location.as_ref().unwrap());
    let open_runtime_path = game_folder.join(
        game_config
            .runtime
            .as_ref()
            .unwrap()
            .as_opposite_local_dll(),
    );
    Ok(if Path::new(&open_runtime_path).exists() {
        fs::remove_file(&open_runtime_path)
            .report()
            .change_context(REvilManagerError::RemoveFileFiled(
                open_runtime_path.display().to_string(),
            ))?;
        debug!(
            "Second runtime file removed {}",
            open_runtime_path.display()
        );
    } else {
        debug!(
            "Second runtime file doesn't exist {}",
            open_runtime_path.display()
        );
    })
}

// #[test]
// fn sort_test {
// REvilManager::so
// }

use std::{cmp::Ordering, collections::HashMap, env, ffi::OsStr, fs, path::Path, process};

use crate::{
    rManager_header::{
        LabelOptions, REvilManager, REvilManagerError, REvilManagerState, REvilThings,
        ResultManagerErr, SORT_DETERMINER,
    },
    refr_github::{ManageGithub, REFRGithub},
    steam::SteamThings,
    tomlConf::{
        config::ConfigProvider,
        configStruct::{ErrorLevel, GameConfig, REvilConfig, Runtime, SteamId},
    },
    unzip::unzip,
    utils::{
        init_logger::init_logger, local_version::LocalFiles, mslink::create_ms_lnk, progress_style,
        version_parser::isRepoVersionNewer,
    },
    DynResult, ARGS, GAMES, GAMES_NEXTGEN_SUPPORT, MAX_ZIP_FILES_PER_GAME_CACHE, NIGHTLY_RELEASE,
    REPO_OWNER, STANDARD_TYPE_QUALIFIER,
};
use dialoguer::{theme::ColorfulTheme, Select};

use error_stack::{IntoReport, Report, Result, ResultExt};
use log::{debug, info, log, trace, warn, Level};
use self_update::update::ReleaseAsset;
use std::time::Duration;

use indicatif::ProgressBar;

impl REvilManager {
    pub fn new(
        config_provider: Box<dyn ConfigProvider>,
        local_provider: Box<dyn LocalFiles>,
        steam_menago: Box<dyn SteamThings>,
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

                        let arguments = format!("--run {}", short_name);
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
                    is_rnewer.map_or((), |is| {
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

        debug!(
            "games_that_require_update, {:?}",
            self.state.games_that_require_update
        );
        Ok(self)
    }

    fn pick_one_game_from_report(&mut self) -> ResultManagerErr<&mut Self> {
        let game_short_name;
        unsafe {
            game_short_name = &ARGS.as_ref().unwrap().run;
        }
        let game_config = self.config.games.get(game_short_name).unwrap();
        let steam_id = game_config.steamId.as_ref().unwrap();
        self.state.selected_game_to_launch = Some(steam_id.to_string());
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
                    does_asset_tdb(short_name, asset)
                        .and_then(|does| {
                            ((does && !nextgen) || (!does && nextgen))
                                .then(|| self.state.selected_assets.push(asset.clone()))
                        })
                        .unwrap_or_else(|| self.state.selected_assets.push(asset.clone()));
                });
                return Some(());
            })
            .unwrap_or_else(|| debug!("Report doesn't contain {} game", game_short_name));

        Ok(self)
    }

    // TODO consider testing scenario for games without NEXTGEN option like i.e. only ["MHRISE". "DCM5", "RE8"]
    fn ask_for_decision(&mut self) -> ResultManagerErr<&mut Self> {
        let (different_found, any_none, games) = &self.prepare_decision_report()?;
        let mut selections = vec![];
        use LabelOptions::*;
        if games.len() > 0 {
            selections.push(UpdateAllGames.to_label());
            if *different_found && !any_none {
                // will choose base of your current local mod settings per game
                selections[0] = UpdateAllGamesAutoDetect.to_label();
            } else if *different_found && *any_none {
                selections.push(UpdateAllGamesPreferStandard.to_label());
                selections[0] = UpdateAllGamesPreferNextgen.to_label();
            }
        } else {
            info!("Not found any games to update");
            return Ok(self);
        };

        let mut texts: Vec<String> = games.keys().cloned().collect();
        texts.sort_by(|a, b| REvilManager::sort(a, b));
        selections.extend(texts);
        debug!("{:#?}", selections);

        let count = self.state.games_that_require_update.len();
        let mut additional_text = "";
        if *different_found && *any_none {
            additional_text = r"Also found that some of your games that
             can support both types Nextgen/Standard don't have mod installed yet.
             Chose which mod type use for them. For other games program will use correct version.";
        }
        selections.push("Skip".to_string());
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("I found {} games that require update. Select which one you want to update or select all. {}", count, additional_text))
            .default(0)
            .items(&selections[..])
            .interact()
            .unwrap();

        debug!(
            "selection {}, different_found {}, any_none {}",
            selection, different_found, any_none
        );

        // important do not change order of below if call as later in iteration may provide out of index error
        let selected_text = &selections[selection];
        let sel = LabelOptions::from(&selected_text[..]);
        match sel {
            Skip => {
                info!("Chosen skip option.");
                return Ok(self);
            }
            _ => (),
        };

        if sel != Skip && sel != Other {
            games.values().for_each(|(asset, include, _)| {
                include
                    .map(|should_include| {
                        if !should_include {
                            debug!("Asset {} not added", asset.name);
                            return;
                        }
                        debug!("adding asset {}", asset.name);
                        self.state.selected_assets.push(asset.clone());
                    })
                    .unwrap_or_else(|| {
                        (*different_found && *any_none).then(|| {
                            if asset.name.contains(STANDARD_TYPE_QUALIFIER) {
                                if sel != UpdateAllGamesPreferStandard {
                                    return;
                                }
                                debug!("adding standard asset for {}", asset.name);
                                self.state.selected_assets.push(asset.clone())
                            } else {
                                if sel != UpdateAllGamesPreferNextgen {
                                    return;
                                }
                                debug!("adding nextgen asset for {}", asset.name);
                                self.state.selected_assets.push(asset.clone());
                            }
                        });
                    });
            });
            return Ok(self);
        }

        if let Some((asset, _, game_id)) = games.get(&selections[selection]) {
            debug!("adding asset {}", asset.name);
            self.state.selected_assets.push(asset.clone().clone());
            self.state.selected_game_to_launch = game_id.clone();
        };
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
                        versions.insert(0, [version.to_string(), asset.name.to_string()].to_vec())
                    })
                    .unwrap_or_else(|| {
                        game_config.versions =
                            Some([[version.to_string(), asset.name.to_string()].to_vec()].to_vec())
                    });

                // set NEXTGEN accordingly to an asset but only for the supported games
                does_asset_tdb(game_short_name, asset)
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
        if self.state.selected_game_to_launch.is_some() {
            return Ok(self);
        }

        let mut selections_h_map: HashMap<String, &SteamId> = HashMap::new();

        self.config.games.iter().for_each(|(short_name, game)| {
            game.versions
                .as_ref()
                .and_then(|versions| {
                    (versions.first().unwrap().len() > 1 && game.runtime.is_some()).then(|| {
                        selections_h_map.insert(
                            format!(
                                "{}: Switch to <{:?}> runtime for {} and run game",
                                SORT_DETERMINER,
                                game.runtime.as_ref().unwrap().as_opposite(),
                                short_name
                            ),
                            game.steamId.as_ref().unwrap(),
                        );
                        ()
                    })
                })
                .unwrap_or_default();
            selections_h_map.insert(
                format!(
                    "Run {} - Runtime <{:?}>",
                    short_name,
                    game.runtime.as_ref().unwrap()
                ),
                game.steamId.as_ref().unwrap(),
            );
        });
        use LabelOptions::*;
        let mut selections: Vec<String> = selections_h_map.keys().cloned().collect();
        selections.sort_by(|a, b| REvilManager::sort(a, b));
        selections.push(LoadDifferentVersionFromCache.to_label());
        selections.push(SwitchType.to_label());
        selections.push(Exit.to_label());
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Select game to run"))
            .default(0)
            .items(&selections[..])
            .interact()
            .unwrap();

        let selected_text = &selections[selection];

        match LabelOptions::from(&selected_text[..]) {
            Exit => {
                info!("Chosen exit option. Bye bye..");
                return Ok(self);
            }
            other => self.state.other_option = Some(other),
        };

        let selected_steam_id = selections_h_map
            .get(&selections[selection])
            .unwrap()
            .clone()
            .to_string();

        if selected_text.contains(SORT_DETERMINER) {
            let (game_short_name, _) = self.find_game_conf_by_steam_id(&selected_steam_id)?;
            let game_short_name = game_short_name.clone();
            let game_config = self.config.games.get_mut(&game_short_name);
            let conf = game_config.unwrap();
            let runtime = conf.runtime.as_ref().unwrap();
            info!(
                "Switching runtime {:?} to {:?} for {}",
                runtime,
                runtime.as_opposite(),
                game_short_name
            );
            conf.runtime = Some(runtime.as_opposite());
        }
        self.state.selected_game_to_launch = Some(selected_steam_id.clone().to_string());
        Ok(self)
    }

    fn ask_for_switch_type_decision(&mut self) -> ResultManagerErr<&mut Self> {
        todo!()
    }

    fn check_for_self_update(&mut self) -> DynResult<&mut Self> {
        todo!()
    }

    fn self_update(&mut self) -> DynResult<&mut Self> {
        todo!()
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

    fn find_game_conf_by_steam_id(
        &self,
        steam_id: &String,
    ) -> ResultManagerErr<(&String, &GameConfig)> {
        let (game_short_name, game_config) = self
            .config
            .games
            .iter()
            .find(|(_, conf)| conf.steamId.as_ref().unwrap() == steam_id)
            .ok_or(Report::new(REvilManagerError::GameNotFoundForGivenSteamId(
                steam_id.to_string(),
            )))?;
        Ok((game_short_name, game_config))
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
    fn prepare_decision_report(
        &self,
    ) -> ResultManagerErr<(
        bool,
        bool,
        HashMap<String, (ReleaseAsset, Option<bool>, Option<String>)>,
    )> {
        // it determines wether you have game that supports different version i.e. RE2 support both nextgen and standard but if you have only game like
        // MHRISE DMC5 then it should not change thus should not display specific message later
        let mut different_found = false;
        // it checks if any nextgen supported game doesn't have nextgen type set - treating like mod is not installed
        let mut any_none = false;
        let mut games: HashMap<String, (ReleaseAsset, Option<bool>, Option<SteamId>)> =
            HashMap::new();
        let report = self
            .github_release_manager
            .as_ref()
            .map(|men| men.getAssetsReport())
            .ok_or(Report::new(
                REvilManagerError::ReleaseManagerIsNotInitialized,
            ))?;
        report.iter().for_each(|(game_short_name, assets)| {
            self.state
                .games_that_require_update
                .contains(game_short_name)
                .then(|| {
                    assets.iter().for_each(|asset| {
                        let mut text = "".to_string();
                        let mut include_for_all_option = Some(true);
                        let game_config = self.config.games.get(game_short_name).unwrap();
                        does_asset_tdb(game_short_name, asset)
                            .and_then(|does| {
                                different_found = true;
                                let nextgen = game_config.nextgen;

                                does.then(|| {
                                    text = format!("{} Standard version", game_short_name);
                                    nextgen.and_then(|nextgen| {
                                        nextgen.then(|| {
                                            include_for_all_option = Some(false);
                                            get_label_for_download_switch(
                                                &mut text, "nextgen", "standard",
                                            );
                                        })
                                    })
                                })
                                .unwrap_or_else(|| {
                                    text = format!("{} Nextgen version", game_short_name);
                                    nextgen.map(|next_gen| {
                                        (!next_gen).then(|| {
                                            include_for_all_option = Some(false);
                                            get_label_for_download_switch(
                                                &mut text, "standard", "nextgen",
                                            );
                                        });
                                    })
                                })
                                .unwrap_or_else(|| {
                                    include_for_all_option = None;
                                    any_none = true;
                                });
                                return Some(());
                            })
                            .unwrap_or_else(|| text = game_short_name.to_string());
                        games.insert(
                            text,
                            (
                                asset.clone(),
                                include_for_all_option,
                                game_config.steamId.clone(),
                            ),
                        );
                    });
                })
                .unwrap_or_default();
        });
        Ok((different_found, any_none, games))
    }
}

fn get_label_for_download_switch(text: &mut String, next_or_std: &str, next_or_std_sec: &str) {
    *text = format!(
        "{}      {}(your current version of mod is {} -> it will switch to {})",
        text,
        SORT_DETERMINER,
        next_or_std.to_string(),
        next_or_std_sec.to_string()
    );
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

fn cleanup_cache(
    manager: &Box<dyn ManageGithub<REFRGithub>>,
    last_ver: &Vec<String>,
    game_short_name: &str,
) -> ResultManagerErr<()> {
    if last_ver.len() < 2 {
        debug!(
            "A Game {} Cache warn: {:?}",
            game_short_name,
            REvilManagerError::CacheNotFoundForGivenVersion(last_ver[0].to_string()).to_string()
        );
        return Ok(());
    }
    let last_ver_nb = &last_ver[0];
    let cache_dir = manager
        .get_local_path_to_cache(Some(&last_ver_nb))
        .or(Err(Report::new(
            REvilManagerError::ReleaseManagerIsNotInitialized,
        )))?;
    Ok(if cache_dir.exists() {
        let file_to_remove = cache_dir.join(last_ver[1].to_string());
        if Path::new(&file_to_remove).exists() {
            fs::remove_file(&file_to_remove).report().change_context(
                REvilManagerError::RemoveZipAssetFromCacheErr(file_to_remove.display().to_string()),
            )?;
        }
        match fs::remove_dir(&cache_dir) {
            Ok(()) => debug!("Directory: {} Removed", cache_dir.display().to_string()),
            Err(err) => debug!(
                "Can not Remove directory: {} Err {}",
                cache_dir.display().to_string(),
                err
            ),
        };
    })
}

// check if asset is TDB or not if it doesn't support nextgen version then None is returned
fn does_asset_tdb(game_short_name: &str, asset: &ReleaseAsset) -> Option<bool> {
    if GAMES_NEXTGEN_SUPPORT.contains(&game_short_name) {
        if asset.name.contains(STANDARD_TYPE_QUALIFIER) {
            return Some(true);
        } else {
            return Some(false);
        }
    }
    None
}

// #[test]
// fn sort_test {
// REvilManager::so
// }

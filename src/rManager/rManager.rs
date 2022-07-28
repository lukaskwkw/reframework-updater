use std::{cmp::Ordering, collections::HashMap, env, ffi::OsStr, fs};

use crate::{
    args::RunAfter,
    dialogs::{
        dialogs::{Ask, DialogsErrors, SwitchActionReport},
        dialogs_label::LabelOptions,
    },
    rManager::cleanup_cache::cleanup_cache,
    rManager::rManager_header::{
        REvilManager, REvilManagerError, REvilManagerState, REvilThings, ResultManagerErr,
        SORT_DETERMINER,
    },
    refr_github::REFRGithub,
    reframework_github::refr_github::ManageGithub,
    steam::SteamThings,
    tomlConf::{
        config::ConfigProvider,
        configStruct::{ErrorLevel, GameConfig, REvilConfig, Runtime, ShortGameName},
    },
    utils::{
        find_game_conf_by_steam_id::find_game_conf_by_steam_id,
        get_local_path_to_cache::get_local_path_to_cache_folder, init_logger::init_logger,
        is_asset_tdb::is_asset_tdb, local_version::LocalFiles, progress_style,
        restart_program::restart_program, version_parser::isRepoVersionNewer,
    },
    DynResult, ARGS, GAMES, MAX_ZIP_FILES_PER_GAME_CACHE, NIGHTLY_RELEASE, REPO_OWNER,
    STANDARD_TYPE_QUALIFIER,
};
use std::path::Path;

#[cfg(test)]
use crate::unzip::unzip::mock_unzip as unzip;
#[cfg(not(test))]
use crate::unzip::unzip::unzip;

use error_stack::{IntoReport, Report, Result, ResultExt};
use log::{debug, error, info, log, trace, warn, Level};
use self_update::update::ReleaseAsset;
use std::time::Duration;

use indicatif::ProgressBar;

use super::rManager_header::AfterUnzipOption;

pub static SWITCH_IDENTIFIER: &str = "switch";

impl REvilManager {
    pub fn new(
        config_provider: Box<dyn ConfigProvider>,
        local_provider: Box<dyn LocalFiles>,
        steam_menago: Box<dyn SteamThings>,
        dialogs: Box<dyn Ask>,
        github_constr: fn(&str, &str) -> Box<dyn ManageGithub>,
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
        info!("config loaded successfully, logger initialized");
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
                    let runtime = game
                        .runtime
                        .clone()
                        .or_else(|| game_config.runtime.clone())
                        .unwrap();
                    debug!("runtime {:?} game {}", runtime, game_short_name);
                    *game = GameConfig {
                        runtime: Some(runtime),
                        nextgen: game.nextgen,
                        runArgs: game.runArgs.clone(),
                        versions: game.versions.clone(),
                        version_in_use: game.version_in_use.clone(),
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

    // TODO maybe divide it into two functions or find better name for function
    fn get_local_settings_per_game_and_amend_current_ones(&mut self) -> &mut Self {
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
            if local_config.runtime.is_some() {
                config.runtime = local_config.runtime;
            }
            // we want check config.versions because maybe found new steam game and we don't want to
            // replace versions information for other games
            if local_config.version.is_some() && config.versions.is_none() {
                let local_ver = local_config.version.unwrap();
                config.versions = Some([[local_ver.to_string()].to_vec()].to_vec());
                config.version_in_use = Some(local_ver);
            }
            config.nextgen = local_config.nextgen;
        }
        pb.finish_with_message("Done");

        trace!("Full config: \n {:#?}", self.config);
        self
    }

    fn generate_ms_links(&mut self) -> ResultManagerErr<&mut Self> {
        let current_exe_path =
            env::current_exe()
                .report()
                .change_context(REvilManagerError::FailedToCreateMsLink(
                    "Env::current_exe fail".to_string(),
                ))?;

        let ms_links_folder = self.local_provider.create_cache_dir()?;

        self.config
            .games
            .iter()
            .try_for_each(|(short_name, _)| -> ResultManagerErr<()> {
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
                self.local_provider
                    .create_ms_lnk(&ms_link_path, &current_exe_path, Some(arguments.clone()))
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
                    })
                    .map(|_| info!("Ms link created for {}", short_name))
                    .unwrap_or_else(|err| {
                        warn!("{}", err);
                        debug!("{:?}", err);
                    });
                Ok(())
            })?;

        Ok(self)
    }

    fn check_for_REFramework_update(&mut self) -> ResultManagerErr<&mut Self> {
        let main = &self.config.main;
        let repo_owner = main
            .repo_owner
            .as_ref()
            .map_or(REPO_OWNER.to_string(), |it| it.to_string());

        let source: String = match &main.chosen_source {
            Some(it) => it.to_string(),
            None => NIGHTLY_RELEASE.to_string(),
        };
        self.github_release_manager = Some((self.refr_ctor)(&repo_owner, &source));

        info!("Checking if new release exists");
        let manager = self
            .github_release_manager
            .as_mut()
            .ok_or_else(|| Report::new(REvilManagerError::ReleaseManagerIsNotInitialized))?;
        manager.get_reframework_latest_release().or_else(|err| {
            Err(Report::new(REvilManagerError::CheckingNewReleaseErr))
                .attach_printable(format!("{:?}", err))
        })?;

        // requires github_release_manager to be initialized
        self.set_games_that_require_update()?;

        info!(
            "games_that_require_update, {:?}",
            self.state.games_that_require_update
        );
        Ok(self)
    }

    fn pick_one_game_from_report_and_set_as_selected(&mut self) -> ResultManagerErr<&mut Self> {
        let game_short_name;
        let should_run_after;
        unsafe {
            game_short_name = &ARGS.as_ref().unwrap().one;
            should_run_after = &ARGS.as_ref().unwrap().run;
        }
        debug!("Args -one {}, -run {:?}", game_short_name, should_run_after);
        let games = &self.config.games;
        let game_config = games.get(game_short_name).unwrap();
        let steam_id = get_steam_id_by_short_name(games, game_short_name);
        if should_run_after.to_bool() {
            self.state.selected_game_to_launch = Some(steam_id.to_string());
        };
        if !self
            .state
            .games_that_require_update
            .contains(game_short_name)
        {
            info!("Update not required for {}", game_short_name);
            return Ok(self);
        }
        set_game_from_report_as_selected_to_download(
            self.github_release_manager.as_ref(),
            self.state.selected_assets.as_mut(),
            game_config,
            game_short_name,
        )?;

        Ok(self)
    }

    fn ask_for_decision(&mut self) -> ResultManagerErr<&mut Self> {
        let report = self
            .github_release_manager
            .as_ref()
            .map(|men| men.getAssetsReport())
            .ok_or_else(|| Report::new(REvilManagerError::ReleaseManagerIsNotInitialized))?;
        self.dialogs
            .ask_for_decision_and_populate_selected_assets(
                &mut self.config,
                &mut self.state,
                report,
            )
            .map_err(|err| match err.current_context() {
                DialogsErrors::NoGamesToUpdate => return REvilManagerError::NoGamesToUpdate,
                _ => return REvilManagerError::Other,
            })?;
        Ok(self)
    }

    fn download_REFramework_update(&mut self) -> ResultManagerErr<&mut Self> {
        let results: Vec<(String, ResultManagerErr<()>)> = self
            .state
            .selected_assets
            .iter()
            .map(|asset| -> (String, ResultManagerErr<()>) {
                let asset_name_result = self
                    .github_release_manager
                    .as_ref()
                    .unwrap()
                    .download_release_asset(asset)
                    .map_err(|err| {
                        Err(Report::new(REvilManagerError::DownloadAssetError(
                            asset.name.clone(),
                        )))
                        .attach_printable(format!("{:?}", err))
                    })
                    .map(|_| (asset.name.to_string(), Ok(())))
                    .unwrap_or_else(|err| (asset.name.to_string(), err));

                asset_name_result
            })
            .collect();

        results.iter().for_each(|(asset_name, result)| {
            result.as_ref().unwrap_or_else(|err| {
                let pos = self
                    .state
                    .selected_assets
                    .iter()
                    .position(|asset| &asset.name == asset_name)
                    .unwrap();
                // if during download error occurs we don't want to do nothing with this asset later
                self.state.selected_assets.remove(pos);
                warn!(
                    "Update error for {} Err: {} Mod has not been updated!",
                    asset_name, err
                );
                debug!("{:?}", err);
                &()
            });
        });
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
        let game_config = self.config.games.get(game_short_name).ok_or_else(|| {
            Report::new(REvilManagerError::GameNotFoundForGivenShortName(
                game_short_name.to_string(),
            ))
        })?;
        let manager = self
            .github_release_manager
            .as_ref()
            .ok_or_else(|| Report::new(REvilManagerError::ReleaseManagerIsNotInitialized))?;
        let release = manager.getRelease();
        let path_to_zip = get_local_path_to_cache_folder(release, version)
            .map(|path| path.join(file_name))
            .map_err(|_| Report::new(REvilManagerError::GetLocalPathToCacheErr))?;
        let location = game_config
            .location
            .as_ref()
            .ok_or_else(|| Report::new(REvilManagerError::GameLocationMissing))?;

        if unzip_skip_fun.is_some() {
            unzip(&path_to_zip, &location, unzip_skip_fun).change_context(
                REvilManagerError::UnzipError(format!(
                    "Couldn't unzip asset {}: for {} game.",
                    path_to_zip.display(),
                    game_short_name
                )),
            )?;
            return Ok(self);
        };
        let closure = game_config
            .runtime
            .as_ref()
            .map(|runtime| -> Box<dyn Fn(&OsStr) -> bool> {
                let should_skip = |f: &OsStr| f == OsStr::new(&runtime.as_opposite_local_dll());
                Box::new(should_skip)
            })
            .unwrap_or_else(|| {
                let should_skip =
                    |f: &OsStr| f == OsStr::new(&Runtime::OpenVR.as_opposite_local_dll());
                Box::new(should_skip)
            });

        unzip(&path_to_zip, location, Some(closure)).change_context(
            REvilManagerError::UnzipError(format!(
                "Couldn't unzip asset {}: for {} game",
                path_to_zip.display(),
                game_short_name
            )),
        )?;
        Ok(self)
    }

    fn unzip_updates(&mut self) -> &mut Self {
        let selected_assets = &self.state.selected_assets;
        let results: Vec<(String, ResultManagerErr<()>)> = selected_assets
            .iter()
            .map(|asset| {
                let game_short_name = match get_game_short_name_from_asset(asset) {
                    Ok(it) => it,
                    Err(err) => {
                        error!("{:#?}", err);
                        return (asset.name.to_string(), Err(err));
                    }
                };

                let game_short_name = game_short_name;
                let asset_name_result = self
                    .unzip_update::<fn(&OsStr) -> bool>(game_short_name, &asset.name, None, None)
                    .map(|_| (asset.name.to_string(), Ok(())))
                    .unwrap_or_else(|err| (asset.name.to_string(), Err(err)));

                asset_name_result
            })
            .collect();

        results.iter().for_each(|(asset_name, result)| {
            result.as_ref().unwrap_or_else(|err| {
                let pos = self
                    .state
                    .selected_assets
                    .iter()
                    .position(|asset| &asset.name == asset_name)
                    .unwrap();

                // if during error occurs we don't want to do nothing with this asset later
                self.state.selected_assets.remove(pos);
                warn!(
                    "Unzip error for {} Err: {} Mod has not been changed!",
                    asset_name, err
                );
                debug!("{:?}", err);
                &()
            });
        });
        self
    }

    fn after_unzip_work(
        &mut self,
        options: Option<Vec<AfterUnzipOption>>,
    ) -> Result<&mut Self, REvilManagerError> {
        let selected_assets = &self.state.selected_assets;
        let manager = self
            .github_release_manager
            .as_ref()
            .ok_or_else(|| Report::new(REvilManagerError::ReleaseManagerIsNotInitialized))?;
        let release = manager.getRelease();
        let version: &str = release
            .as_ref()
            .ok_or_else(|| Report::new(REvilManagerError::ReleaseIsEmpty))?
            .name
            .as_ref();
        let results: Vec<ResultManagerErr<()>> = selected_assets
            .iter()
            .map(|asset| -> ResultManagerErr<()> {
                let game_short_name = get_game_short_name_from_asset(asset)?;
                info!("After unzip work for {} - start", game_short_name);

                if options.is_none()
                    || options.is_some()
                        && !options.as_ref().unwrap().iter().any(|option| {
                            *option == AfterUnzipOption::SkipRemovingFromRequiredUpdates
                        })
                {
                    // remove game from req_update_games vec as it is already updated!
                    remove_game_from_update_needed_ones(
                        self.state.games_that_require_update.as_mut(),
                        game_short_name,
                    );
                }

                let game_config = self.config.games.get_mut(game_short_name).ok_or_else(|| {
                    Report::new(REvilManagerError::GameNotFoundForGivenShortName(
                        game_short_name.to_string(),
                    ))
                })?;

                // add version from asset to array or create new array with the asset version
                if options.is_none()
                    || options.is_some()
                        && !options
                            .as_ref()
                            .unwrap()
                            .iter()
                            .any(|option| *option == AfterUnzipOption::SkipSettingVersion)
                {
                    add_asset_ver_to_game_conf_ver(game_config, version, asset);
                }

                // set NEXTGEN accordingly to an asset but only for the supported games
                match is_asset_tdb(game_short_name, asset) {
                    Some(is_tdb) => game_config.nextgen = Some(!is_tdb),
                    None => (),
                };

                // remove second, not needed runtime file as for example when switching between different runtime versions
                // second file may persists therefore blocking loading OpenXR runtime from loading
                remove_second_runtime_file(game_config)?;

                // it is ok to unwrap as in add_asset_ver_to_game_conf_ver step we added array to that game config
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
                info!("After unzip work for {game_short_name} - done");
                Ok(())
            })
            .collect();

        results.iter().for_each(|result| {
            result.as_ref().unwrap_or_else(|err| {
                warn!("{}", err);
                debug!("{:#?}", err);
                &()
            });
        });
        self.state.selected_assets.drain(..);

        Ok(self)
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
            .ask_for_game_decision_if_needed_and_set_game_to_launch(
                &mut self.config,
                &mut self.state,
            )
            .change_context(REvilManagerError::Other)?;
        Ok(self)
    }
    fn ask_for_switch_runtime_if_needed(&mut self) -> ResultManagerErr<&mut Self> {
        use LabelOptions::*;
        let selected_option = self.state.selected_option.as_ref();
        if selected_option.is_none()
            || selected_option.is_some() && selected_option.unwrap() != &SwitchRuntimeSection
        {
            return Ok(self);
        }
        self.dialogs
            .ask_for_runtime_decision_and_change_it(&mut self.config, &mut self.state)
            .change_context(REvilManagerError::Other)?;
        self.save_config()?;
        Ok(self)
    }

    fn ask_for_switch_type_decision(&mut self, run_after: RunAfter) -> ResultManagerErr<&mut Self> {
        let selected_option = self.state.selected_option.as_ref();
        if selected_option.is_none()
            || selected_option.is_some() && selected_option.unwrap() != &LabelOptions::SwitchType
        {
            return Ok(self);
        }
        let what_next = self
            .dialogs
            .get_switch_type_decision(&mut self.config, &mut self.state)
            .change_context(REvilManagerError::Other)?;

        use SwitchActionReport::*;
        match what_next {
            ToggleNUnzipSave(short_name, second_asset_name) => {
                self.toggle_nextgen(&short_name);
                self.unzip_update::<fn(&OsStr) -> bool>(
                    &short_name,
                    &second_asset_name,
                    None,
                    None,
                )?;
                // it is required to populate selected_assets for after_unzip_work
                self.state.selected_assets.push(ReleaseAsset {
                    download_url: "".to_string(),
                    name: second_asset_name,
                });
                self.after_unzip_work(Some(
                    [
                        AfterUnzipOption::SkipSettingVersion,
                        AfterUnzipOption::SkipRemovingFromRequiredUpdates,
                    ]
                    .to_vec(),
                ))?;
                self.save_config()?;
                self.state.selected_option = Some(LabelOptions::Back);
                return Ok(self);
            }
            ToggleNSetSwitchSaveRestart(short_name) => {
                self.toggle_nextgen(&short_name);
                self.set_switch_as_version(&short_name);
                self.save_config()?;
                restart_program(run_after, short_name)
                    .report()
                    .change_context(REvilManagerError::ErrorRestartingProgram)?;
            }
            UnsetNonExistentToggleNRestart(game_short_name, second_asset_name) => {
                self.toggle_nextgen(&game_short_name);
                let game_conf = self.config.games.get_mut(&game_short_name).unwrap();
                let first_set = game_conf.versions.as_mut().unwrap().first_mut().unwrap();
                let position = first_set
                    .iter()
                    .skip(1)
                    .position(|asset_name| *asset_name == second_asset_name)
                    .unwrap();
                first_set.remove(position);

                self.set_switch_as_version(&game_short_name);
                self.save_config()?;
                restart_program(run_after, game_short_name)
                    .report()
                    .change_context(REvilManagerError::ErrorRestartingProgram)?;
            }
            Early => {
                self.state.selected_option = Some(LabelOptions::Back);
                return Ok(self);
            }
            ToggleNSaveRestart(game_short_name) => {
                self.toggle_nextgen(&game_short_name);
                self.save_config()?;
                restart_program(run_after, game_short_name)
                    .report()
                    .change_context(REvilManagerError::ErrorRestartingProgram)?;
            }
        }
        Ok(self)
    }

    fn load_from_cache_if_chosen(&mut self) -> ResultManagerErr<&mut Self> {
        use LabelOptions::*;
        let selected_option = self.state.selected_option.as_ref();
        if selected_option.is_none()
            || selected_option.is_some()
                && selected_option.unwrap() != &LoadDifferentVersionFromCache
        {
            return Ok(self);
        }
        let option = self.dialogs.get_selected_cache_option(&self.config);
        debug!("Ask for cache return option {:#?}", option);
        match option {
            LoadFromCache(short_name, asset_name, version) => {
                debug!(
                    "short_name- {} asset_name- {} version-{}",
                    short_name, asset_name, version
                );
                self.unzip_update::<fn(&OsStr) -> bool>(
                    &short_name,
                    &asset_name,
                    Some(&version),
                    None,
                )?;
                // it is required to populate selected_assets for after_unzip_work
                self.state.selected_assets.push(ReleaseAsset {
                    download_url: "".to_string(),
                    name: asset_name,
                });
                self.after_unzip_work(Some(
                    [
                        AfterUnzipOption::SkipSettingVersion,
                        AfterUnzipOption::SkipRemovingFromRequiredUpdates,
                    ]
                    .to_vec(),
                ))?;
                match self.config.games.get_mut(&short_name) {
                    Some(game_config) => game_config.version_in_use = Some(version),
                    None => (),
                };
                self.state.selected_option = Some(LabelOptions::Back);
            }
            Back => {
                self.state.selected_option = Some(Back);
            }
            _ => {}
        }
        Ok(self)
    }

    fn check_for_self_update(&mut self) -> DynResult<&mut Self> {
        todo!()
    }

    fn self_update(&mut self) -> DynResult<&mut Self> {
        todo!()
    }

    fn before_launch_procedure(&self, steam_id: &String) -> ResultManagerErr<String> {
        let (game_short_name, game_config) = find_game_conf_by_steam_id(&self.config, steam_id)?;
        info!("Before launch procedure - start");
        if game_config.runtime.is_none() {
            warn!("Mod runtime for {} is None", game_short_name);
            return Ok(game_short_name.to_string());
        }
        if game_config.versions.is_none() || game_config.version_in_use.is_none() {
            error!(
                "Do you have mod installed for? {} Please select one to download",
                game_short_name
            );
            return Err(Report::new(REvilManagerError::ModIsNotInstalled(
                game_short_name.to_string(),
            )));
        }
        let maybe_vec = game_config
            .versions
            .as_ref()
            .unwrap()
            .iter()
            .find(|ver_set| {
                ver_set.first().unwrap() == game_config.version_in_use.as_ref().unwrap()
            });

        let mut version_vec: &Vec<String> = &Vec::new();
        if maybe_vec.is_none() {
            warn!("Your version is not in cache anymore. Will try to get runtime from latest instead.");
            version_vec = game_config.versions.as_ref().unwrap().first().unwrap();
        } else {
            info!(
                "Checking runtime for {} version",
                game_config.version_in_use.as_ref().unwrap()
            );
            version_vec = maybe_vec.unwrap();
        }
        if version_vec.len() < 2 {
            debug!("Mod version has no cache file");
            return Ok(game_short_name.to_string());
        }
        let game_dir = game_config
            .location
            .as_ref()
            .ok_or_else(|| Report::new(REvilManagerError::GameLocationMissing))?;
        let game_dir = Path::new(&game_dir);

        let runtime = game_config.runtime.as_ref().unwrap();
        if !game_dir.join(runtime.as_local_dll()).exists() {
            let should_skip_all_except = |file: &OsStr| file != OsStr::new(&runtime.as_local_dll());
            let ver = &version_vec[0];

            let file_name = version_vec.iter().skip(1).find(|name| {
                is_asset_tdb(
                    game_short_name,
                    &ReleaseAsset {
                        name: name.to_string(),
                        ..Default::default()
                    },
                )
                .and_then(|is_tdb| game_config.nextgen.map(|nextgen| (is_tdb, nextgen)))
                .map(|(is_tdb, nextgen)| (is_tdb && !nextgen) || (!is_tdb && nextgen)) // if asset is none TDB/NG or nextgen field is missing then just return 1 st item
                .unwrap_or(true)
            });

            // TODO should be safe to unwrap below but maybe some tests?
            let file_name = file_name.unwrap();

            self.unzip_update(
                game_short_name,
                file_name,
                Some(ver),
                Some(should_skip_all_except),
            )?;
            info!("Unzipped only {} file", runtime.as_local_dll());
        }

        remove_second_runtime_file(game_config)?;

        info!("Before launch procedure - end");
        Ok(game_short_name.to_string())
    }

    fn launch_game(&mut self) -> ResultManagerErr<&mut Self> {
        if let Some(steam_id) = &self.state.selected_game_to_launch {
            let game_short_name = self.before_launch_procedure(steam_id)?;

            info!("Launching the game {}", game_short_name);
            self.steam_menago
                .run_game_via_steam_manager(steam_id)
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

    fn decision_loop(&mut self) -> ResultManagerErr<&mut Self> {
        while self.state.selected_option == Some(LabelOptions::GoTop)
            && self.state.selected_game_to_launch.is_none()
        {
            debug!("Select download option");
            self.ask_for_decision()
                .and_then(|this| this.download_REFramework_update())
                .and_then(|this| this.unzip_updates().after_unzip_work(None))
                .and_then(|this| this.save_config())
                .map(|_| Ok(()))
                .unwrap_or_else(|err| {
                    if err.current_context() == &REvilManagerError::NoGamesToUpdate {
                        return Ok(());
                    }
                    Err(err)
                })?;

            self.state.selected_option = Some(LabelOptions::Back);

            while self.state.selected_option == Some(LabelOptions::Back)
                && self.state.selected_game_to_launch.is_none()
            {
                debug!("Select decision");
                info!(
                    "Latest mod version: {}",
                    self.github_release_manager
                        .as_ref()
                        .and_then(|man| man.getRelease())
                        .map(|release| release.name.to_string())
                        .unwrap_or_default()
                );
                self.ask_for_game_decision_if_needed()
                    .and_then(|this| this.ask_for_switch_type_decision(RunAfter::no))
                    .and_then(|this| this.load_from_cache_if_chosen())
                    .and_then(|this| this.ask_for_switch_runtime_if_needed())?;
            }
        }
        Ok(self)
    }
}

impl REvilManager {
    fn toggle_nextgen(&mut self, short_name: &String) {
        let game_conf = self.config.games.get_mut(short_name).unwrap();
        let nextgen = game_conf.nextgen.as_mut().unwrap();
        *nextgen = !*nextgen;
    }

    fn set_switch_as_version(&mut self, short_name: &String) {
        let versions = self
            .config
            .games
            .get_mut(short_name)
            .unwrap()
            .versions
            .as_mut()
            .unwrap();
        let first_set = versions.first_mut().unwrap();
        let version = first_set.first_mut().unwrap();
        *version = SWITCH_IDENTIFIER.to_string();
    }

    fn set_games_that_require_update(&mut self) -> ResultManagerErr<()> {
        let manager = self
            .github_release_manager
            .as_mut()
            .ok_or_else(|| Report::new(REvilManagerError::ReleaseManagerIsNotInitialized))?;
        let release = manager.getRelease();
        self.config
            .games
            .iter()
            .try_for_each(|(short_name, game)| -> ResultManagerErr<()> {
                if game.versions.is_some() {
                    let latest_local_version = game.versions.as_ref().unwrap().first().unwrap();
                    let latest_github_version = release
                        .as_ref()
                        .ok_or_else(|| Report::new(REvilManagerError::ReleaseIsEmpty))?
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
                    if let Some(is) = is_rnewer {
                        is.then(|| {
                            self.state
                                .games_that_require_update
                                .push(short_name.to_string())
                        })
                        .unwrap_or(());
                    }
                } else {
                    debug!(
                        "Version is None treating like needs to be added for update. For {}.",
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

fn set_game_from_report_as_selected_to_download(
    github_release_manager: Option<&Box<dyn ManageGithub<REFRGithub>>>,
    selected_assets: &mut Vec<ReleaseAsset>,
    game_config: &GameConfig,
    game_short_name: &String,
) -> ResultManagerErr<()> {
    let rel_manager = github_release_manager.as_ref();
    let rel_manager = rel_manager
        .ok_or_else(|| Report::new(REvilManagerError::ReleaseManagerIsNotInitialized))?;
    let assets_report = rel_manager.getAssetsReport();

    assets_report
        .iter()
        .find(|(short_name, _)| *short_name == game_short_name)
        .and_then(|(short_name, assets)| {
            assets.iter().for_each(|asset| {
                let should_include_if_ng_supported = is_asset_tdb(short_name, asset)
                    .and_then(|is_tdb| {
                        if game_config.nextgen.is_some() {
                            return Some((is_tdb, game_config.nextgen.unwrap()));
                        };
                        // if nextgen field is missing but game supports both versions then download TDB
                        return Some((is_tdb, false));
                    })
                    .and_then(|(is_tdb, nextgen)| {
                        Some((is_tdb && !nextgen) || (!is_tdb && nextgen))
                    });

                if should_include_if_ng_supported.is_some()
                    && should_include_if_ng_supported.unwrap()
                {
                    debug!("TDB/Nextgen. Added asset to download: {}", asset.name);
                    selected_assets.push(asset.clone())
                } else if should_include_if_ng_supported.is_none() {
                    debug!(
                        "None-TDB/None-Nextgen Added asset to download: {}",
                        asset.name
                    );
                    selected_assets.push(asset.clone())
                }
            });
            return Some(());
        })
        .unwrap_or_else(|| error!("Report doesn't contain {} game", game_short_name));
    Ok(())
}

fn remove_game_from_update_needed_ones(req_update_games: &mut Vec<String>, game_short_name: &str) {
    match req_update_games.iter().position(|sn| sn == game_short_name) {
        Some(pos) => req_update_games.remove(pos),
        None => return (),
    };
}

fn get_game_short_name_from_asset(asset: &ReleaseAsset) -> ResultManagerErr<&str> {
    let game_short_name = asset
        .name
        .split_once(STANDARD_TYPE_QUALIFIER)
        .map(|(short_name, _)| short_name)
        .or_else(|| {
            asset
                .name
                .split_once(".zip")
                .map(|(short_name, _)| short_name)
        })
        .ok_or_else(|| {
            Report::new(REvilManagerError::CannotDeductShortNameFromAssetName(
                asset.name.clone(),
            ))
        })?;
    Ok(game_short_name)
}

fn add_asset_ver_to_game_conf_ver(
    game_config: &mut GameConfig,
    version: &str,
    asset: &ReleaseAsset,
) {
    debug!("Adding asset {}", &asset.name);
    let version_and_switch = game_config.versions.as_ref().map(|versions| {
        let first_set = versions.first().unwrap();
        if first_set[0] == SWITCH_IDENTIFIER {
            if first_set.len() > 1 {
                let vecc = [
                    version.to_string(),
                    asset.name.to_string(),
                    first_set[1].to_string(),
                ]
                .to_vec();
                debug!(
                    "switch has one or more assets. Assets len {}",
                    first_set.len() - 1
                );
                (vecc, true)
            } else {
                debug!("switch has no assets");
                ([version.to_string(), asset.name.to_string()].to_vec(), true)
            }
        } else {
            debug!("no switch asset {}", asset.name);
            (
                [version.to_string(), asset.name.to_string()].to_vec(),
                false,
            )
        }
    });
    if let Some(version_and_switch) = version_and_switch {
        let (version, switch) = version_and_switch;
        let versions = game_config.versions.as_mut().unwrap();

        if switch {
            versions.remove(0);
            versions.insert(0, version);
        } else {
            versions.insert(0, version);
        }
    } else {
        game_config.versions =
            Some([[version.to_string(), asset.name.to_string()].to_vec()].to_vec());
    }
    game_config.version_in_use = Some(version.to_string());
}

fn remove_second_runtime_file(game_config: &GameConfig) -> ResultManagerErr<()> {
    let game_folder = Path::new(
        game_config
            .location
            .as_ref()
            .ok_or_else(|| Report::new(REvilManagerError::GameLocationMissing))?,
    );
    let open_runtime_path = game_folder.join(
        game_config
            .runtime
            .as_ref()
            .ok_or_else(|| REvilManagerError::ModRuntimeIsNone("".to_string()))?
            .as_opposite_local_dll(),
    );
    if Path::new(&open_runtime_path).exists() {
        fs::remove_file(&open_runtime_path)
            .report()
            .change_context(REvilManagerError::RemoveFileFailed(
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
    };
    Ok(())
}

fn get_steam_id_by_short_name<'a>(
    games: &'a HashMap<ShortGameName, GameConfig>,
    game_short_name: &'a String,
) -> &'a String {
    // TODO maybe handle those two unwraps?
    let game_config = games.get(game_short_name).unwrap();
    let steam_id = game_config.steamId.as_ref().unwrap();
    steam_id
}
#[cfg(test)]
pub mod tests {
    use crate::{
        args::ArgsClap,
        tests::{
            config_provider_mock::mock_conf_provider::load_from_file_default_return_mock,
            manager_mocks::init_manager_mocks,
        },
    };

    use super::*;

    fn init() -> REvilManager {
        let (
            steam_menago,
            local_provider_mock,
            dialogs,
            mut config_provider_mock,
            _ctx,
            mock_reft_constr,
        ) = init_manager_mocks();
        config_provider_mock
            .expect_load_from_file()
            .returning(load_from_file_default_return_mock());
        let mut evil_manager = REvilManager::new(
            config_provider_mock,
            local_provider_mock,
            steam_menago,
            dialogs,
            mock_reft_constr,
        );
        evil_manager.github_release_manager = Some((evil_manager.refr_ctor)("praydog", "nightly"));
        evil_manager.load_config().unwrap();
        evil_manager
    }

    #[test]
    fn pick_one_game_from_report_and_set_as_selected_for_nextgen_tdb_game() {
        ["RE2", "RE3"].to_vec().iter().for_each(|short_name| {
            unsafe {
                ARGS = Some(ArgsClap {
                    level: ErrorLevel::info,
                    one: short_name.to_string(),
                    run: RunAfter::yes,
                });
            }
            let mut evil_manager = init();
            evil_manager
                .state
                .games_that_require_update
                .push(short_name.to_string());
            evil_manager
                .pick_one_game_from_report_and_set_as_selected()
                .unwrap();

            if short_name == &"RE2" {
                assert_eq!(evil_manager.state.selected_assets[0].name, "RE2.zip");
            } else if short_name == &"RE3" {
                assert_eq!(evil_manager.state.selected_assets[0].name, "RE3_TDBXXX.zip");
            }
        });
    }

    #[test]
    fn pick_one_game_from_report_and_set_as_selected_for_single_type_game() {
        unsafe {
            ARGS = Some(ArgsClap {
                level: ErrorLevel::info,
                one: "RE8".to_string(),
                run: RunAfter::yes,
            });
        }
        let mut evil_manager = init();
        evil_manager
            .state
            .games_that_require_update
            .push("RE8".to_string());

        // config mock is missing steam id for RE8 on purpose so adding manually here
        evil_manager.config.games.get_mut("RE8").unwrap().steamId = Some("1196590".to_string());
        evil_manager
            .pick_one_game_from_report_and_set_as_selected()
            .unwrap();
        assert_eq!(evil_manager.state.selected_assets[0].name, "RE8.zip");
    }
}

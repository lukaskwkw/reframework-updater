#[cfg(test)]
use mockall::automock;

use std::{collections::HashMap, error::Error};

use dialoguer::{theme::ColorfulTheme, Select};
use error_stack::{Report, Result, ResultExt};
use log::{debug, info, warn};
use self_update::update::ReleaseAsset;

use crate::{
    dialogs::dialogs_label::{LabelOptions, SWITCH_RUNTIME_PART},
    rManager::rManager_header::{
        REvilManager, REvilManagerState, ResultManagerErr, SORT_DETERMINER,
    },
    tomlConf::configStruct::{REvilConfig, ShortGameName, SteamId},
    utils::{
        find_game_conf_by_steam_id::find_game_conf_by_steam_id,
        get_local_path_to_cache::get_local_path_to_cache_folder, is_asset_tdb::is_asset_tdb,
    },
    STANDARD_TYPE_QUALIFIER, reframework_github::refr_github::AssetsReport, GAMES_NEXTGEN_SUPPORT,
};

#[derive(Debug, Default)]
pub enum DialogsErrors {
    #[default]
    Other,
    GameNotFoundForGivenSteamId(String),
    NoGamesToUpdate,
}
// TODO Fill above structs with more errors rather than just using Other everywhere.

impl std::fmt::Display for DialogsErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DialogsErrors")
    }
}

impl Error for DialogsErrors {}

pub type ResultDialogsErr<T> = Result<T, DialogsErrors>;

#[cfg_attr(test, automock)]
pub trait Ask {
    fn ask_for_runtime_decision_and_change_it(&mut self,
        config: &mut REvilConfig,
        state: &mut REvilManagerState
    ) -> ResultDialogsErr<()>;
    fn ask_for_decision_and_populate_selected_assets(
        &mut self,
        config: &mut REvilConfig,
        state: &mut REvilManagerState,
        report: &HashMap<String, Vec<ReleaseAsset>>,
    ) -> ResultDialogsErr<()>;
    fn main_section(
        &mut self,
        config: &mut REvilConfig,
        state: &mut REvilManagerState,
    ) -> ResultDialogsErr<()>;
    fn get_selected_cache_option(&mut self, config: &REvilConfig) -> LabelOptions;
    fn get_switch_type_decision(
        &mut self,
        config: &REvilConfig,
        state: &REvilManagerState,
    ) -> ResultDialogsErr<SwitchActionReport>;
}

pub struct Dialogs;

type SecondAssetName = String;
pub enum SwitchActionReport {
    ToggleNSaveRestart(ShortGameName),
    ToggleNUnzipSave(ShortGameName, SecondAssetName),
    UnsetNonExistentToggleNRestart(ShortGameName, SecondAssetName),
    ToggleNSetSwitchSaveRestart(ShortGameName),
    Early,
}
use LabelOptions::*;
impl Ask for Dialogs {
    fn ask_for_decision_and_populate_selected_assets(
        &mut self,
        config: &mut REvilConfig,
        state: &mut REvilManagerState,
        report: &HashMap<String, Vec<ReleaseAsset>>,
    ) -> ResultDialogsErr<()> {
        let (different_found, any_not_installed_mods_with_both_ver_supporting, game_decisions) = &self
            .prepare_decision_report(config, state, report)
            .change_context(DialogsErrors::Other)?;
        let mut selections = vec![];
        if populate_selections_with_general_options(game_decisions, &mut selections, different_found, any_not_installed_mods_with_both_ver_supporting).is_none() {
            info!("Not found any games to update");
            return Err(Report::new(DialogsErrors::NoGamesToUpdate));
        }

        let mut texts: Vec<String> = game_decisions.keys().cloned().collect();
        texts.sort_by(|a, b| REvilManager::sort(a, b));
        selections.extend(texts);
        debug!("{:#?}", selections);

        let count = state.games_that_require_update.len();
        let mut additional_text = "";
        if *different_found && *any_not_installed_mods_with_both_ver_supporting {
            additional_text = r"Also found that some of your games that
             can support both types Nextgen/Standard don't have mod installed yet.
             Chose which mod type use for them. For other games program will use correct version.";
        }
        selections.push(Skip.to_label());
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("I found {} games that require update. Select which one you want to update or select all. {}", count, additional_text))
            .default(0)
            .items(&selections[..])
            .interact()
            .unwrap();

        debug!(
            "selection {}, different_found {}, any_none {}",
            selection, different_found, any_not_installed_mods_with_both_ver_supporting
        );

        // important do not change order of below if call as later in iteration may provide out of index error
        let selected_text = &selections[selection];
        let sel = LabelOptions::from(&selected_text[..]);
        if sel == Skip {
            info!("Chosen skip option.");
            return Ok(());
        }

        if populate_selected_assets_base_on_general_option(sel, game_decisions, state, different_found, any_not_installed_mods_with_both_ver_supporting).is_some() {
            return Ok(());
        }

        if let Some((asset, _, game_id)) = game_decisions.get(&selections[selection]) {
            debug!("Adding single asset {}", asset.name);
            state.selected_assets.push(asset.clone());
        };
        Ok(())
    }

    fn main_section(
        &mut self,
        config: &mut REvilConfig,
        state: &mut REvilManagerState,
    ) -> ResultDialogsErr<()> {
        if state.selected_game_to_launch.is_some() {
            return Ok(());
        }

        let mut selections_h_map: HashMap<String, &SteamId> = HashMap::new();
        let mut any_game_that_support_2_versions = false;
        config.games.iter().for_each(|(short_name, game)| {
            if !any_game_that_support_2_versions && GAMES_NEXTGEN_SUPPORT.contains(&&short_name[..]) {
                any_game_that_support_2_versions = true;
            }
            let ver_in_use = game.version_in_use.as_ref().map(|ver| ver.to_string()).unwrap_or_default();
            // TODO below should add <cache> string to game if it is loaded from cache but should be fixed
            //      as for now it will show <cache> for games that only has one version or version_in_use is not later in array
            //      there should be first check versions.any() if version_in_use is later in array and then check for first
            // let version_part = game.versions.as_ref()
            //     .and_then(|versions| versions.first())
            //     .and_then(|first_set| first_set.first())
            //     .map(|ver| ver == &ver_in_use)
            //     .and_then(|is_same_as_latest| is_same_as_latest.then(|| ver_in_use.clone()))
            //     .unwrap_or_else(|| format!("{} <cache>", ver_in_use.clone()));

           selections_h_map.insert(
                format!(
                    "Run {}{} - <{:?}> {}",
                    short_name,
                    game.nextgen.map(|nextgen| {
                        if nextgen {
                            " <Nextgen>"
                        } else {
                            " <Standard>"
                        }
                    }).unwrap_or_default(),
                    game.runtime.as_ref().unwrap(),
                    ver_in_use
                ),
                game.steamId.as_ref().unwrap(),
            );
        });
        let mut selections: Vec<String> = selections_h_map.keys().cloned().collect();
        selections.push(SwitchRuntimeSection.to_label());
        selections.push(LoadDifferentVersionFromCache.to_label());
        if any_game_that_support_2_versions {
            selections.push(SwitchType.to_label());
        }
        selections.push(RescanLocal.to_label());
        selections.push(GoTop.to_label());
        selections.push(Exit.to_label());

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select game to run".to_string())
            .default(0)
            .items(&selections[..])
            .interact()
            .unwrap();

        let selected_text = &selections[selection];

        match LabelOptions::from(&selected_text[..]) {
            SwitchRuntimeSection => {
                state.selected_option = Some(SwitchRuntimeSection);
                return Ok(());
            }
            Exit => {
                info!("Chosen exit option. Bye bye..");
                state.selected_option = Some(Exit);
                return Ok(());
            }
            SwitchType => {
                state.selected_option = Some(SwitchType);
                return Ok(());
            }
            LoadDifferentVersionFromCache => {
                state.selected_option = Some(LoadDifferentVersionFromCache);
                return Ok(());
            },
            GoTop => {
                state.selected_option = Some(GoTop);
                return Ok(());
            }
            RescanLocal => {
                state.selected_option = Some(RescanLocal);
                return Ok(());
            }
            _ => (),
        };

        let selected_steam_id = selections_h_map
            .get(&selections[selection])
            .unwrap()
            .clone()
            .to_string();

        state.selected_game_to_launch = Some(selected_steam_id);
        Ok(())
    }

    fn get_selected_cache_option(&mut self, config: &REvilConfig) -> LabelOptions {
        let mut selections: Vec<String> = Vec::new();
        config.games.iter().for_each(|(short_name, game_config)| {
            let versions = game_config.versions.as_ref();
            if versions.is_none() {
                info!("Not found any version records for {}. Please download one first", short_name);
                return;
            }
            let versions = versions.unwrap();
            for ver_set in versions.iter() {
                if ver_set.len() < 2 {
                    continue;
                }
                let ver = ver_set.first().unwrap();
                let mut label_appendix: String = "".to_string();
                if game_config
                    .version_in_use
                    .as_ref()
                    .map(|ver_in_use| ver_in_use == ver)
                    .unwrap_or_default()
                {
                    label_appendix = format!("{SORT_DETERMINER} this is your current version - ");
                }
                ver_set.iter().skip(1).for_each(|asset_name| {
                    match get_local_path_to_cache_folder(None, Some(ver)) {
                        Ok(folder) => {
                            if !folder.join(asset_name).exists() {
                                return;
                            }
                        }
                        Err(err) => {
                            warn!("{}", err);
                            debug!("{:#?}", err);
                            return;
                        }
                    };
                    let label = LoadFromCache(
                        short_name.to_string(),
                        asset_name.to_string(),
                        ver.to_string(),
                    )
                    .to_label();
                    let label = format!("{}{}", label_appendix, label);
                    selections.push(label);
                })
            }
        });
        selections.sort_by(|a, b| REvilManager::sort(a, b));
        selections.push(Back.to_label());
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(r"Select game and its mod version to switch. 
                Note TDB = standard version if game supports standard/nextgen 
                where non TDB = Nextgen version ".to_string())
            .default(0)
            .items(&selections[..])
            .interact()
            .unwrap();
        let selected_text = &selections[selection];

        LabelOptions::from(&selected_text[..])
    }

    fn get_switch_type_decision(
        &mut self,
        config: &REvilConfig,
        state: &REvilManagerState,
    ) -> ResultDialogsErr<SwitchActionReport> {
        use SwitchActionReport::*;
        let mut selections: Vec<String> = config
            .games
            .iter()
            .filter_map(|(short_name, game)| {
                if !GAMES_NEXTGEN_SUPPORT.contains(&&short_name[..]) {
                    debug!("Game doesn't support both versions {}", short_name);
                    return None;
                }
                if game.versions.is_none() || game.version_in_use.is_none() {
                    info!("Is mod installed for {}?", short_name);
                    return None;
                }
                if game.versions.as_ref().unwrap().first().unwrap().first().unwrap() != game.version_in_use.as_ref().unwrap() {
                    info!(r"Switch type decision only supports latest cached versions.
                     If you want to switch to older version then use load from cache and select appropriate one.
                      Game {}", short_name);
                    return None;
                }
                let label = game.nextgen.map(|nextgen| {
                    if nextgen {
                        SwitchToStandard(short_name.to_string()).to_label()
                    } else {
                        SwitchToNextgen(short_name.to_string()).to_label()
                    }
                });
                label
            })
            .collect();

        selections.push(Back.to_label());
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select game to switch".to_string())
            .default(0)
            .items(&selections[..])
            .interact()
            .unwrap();
        let selected_text = &selections[selection];

        match LabelOptions::from(&selected_text[..]) {
            Back => {
                return Ok(Early);
            }
            SwitchToStandard(short_name) | SwitchToNextgen(short_name) => {
                debug!("Selected -> {:#?}", short_name);
                let game_config = config.games.get(&short_name).unwrap();
                
                if !state.games_that_require_update.contains(&short_name) {
                    let next_gen = game_config.nextgen.unwrap();
                    let versions = game_config.versions.as_ref().unwrap();
                    let first_set = versions.first().unwrap();

                    let second_asset_name = first_set.iter().skip(1).find(|name| {
                        is_asset_tdb(
                            &short_name,
                            &ReleaseAsset {
                                name: name.to_string(),
                                ..Default::default()
                            },
                        )
                        // we want other type so condition below has to be different than usually
                        .map(|is_tdb| ((is_tdb && next_gen) || (!is_tdb && !next_gen)))
                        .unwrap_or_default()
                    });

                    if let Some(second_asset_name) = second_asset_name {
                        debug!("preparing unzip for {}", second_asset_name);
                        let path_to_zip = get_local_path_to_cache_folder(None, Some(&first_set[0]))
                            .map(|path| path.join(second_asset_name))
                            .map_err(|_| Report::new(DialogsErrors::Other))?;
                        if !path_to_zip.exists() {
                            return Ok(UnsetNonExistentToggleNRestart(short_name, second_asset_name.to_string()));
                        }
                        return Ok(ToggleNUnzipSave(short_name, second_asset_name.clone()));
                    }

                } else {
                    debug!("Game {} requires update anyway", short_name);
                    return Ok(ToggleNSaveRestart(short_name));
                }
                
                return Ok(ToggleNSetSwitchSaveRestart(short_name));
            }
            _ => (),
        };
        Ok(Early)
    }

    fn ask_for_runtime_decision_and_change_it(&mut self, config: &mut REvilConfig, state: &mut REvilManagerState) -> ResultDialogsErr<()> {
        let mut selections_h_map: HashMap<String, &SteamId> = HashMap::new();
        config.games.iter().for_each(|(short_name, game)| {
        game.versions
                .as_ref()
                .and_then(|versions| {
                    // TODO now if loaded from cache and if latest version doesn't have zip assets in vector even when version_in_use is different and has, game won't show on list
                    //      to fix this we need to replace this condition here with searching in array for cache
                    //      tip: check rManager->rescan_option for find and use game.version_in_use to comparison
                    (versions.first().unwrap().len() > 1 && game.runtime.is_some()).then(|| {
                        selections_h_map.insert(
                            format!(
                                "{} <{:?}> for {}",
                                SWITCH_RUNTIME_PART,
                                game.runtime.as_ref().unwrap().as_opposite(),
                                short_name
                            ),
                            game.steamId.as_ref().unwrap(),
                        );
                        
                    })
                })
                .unwrap_or_default();
            });
        if selections_h_map.is_empty() {
            info!("Please download mod for particular game to be able to switch between runtime");

        }
        let mut selections: Vec<String> = selections_h_map.keys().cloned().collect();
        selections.push(Back.to_label());

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select game to run".to_string())
            .default(0)
            .items(&selections[..])
            .interact()
            .unwrap();

        let selected_text = &selections[selection];

        match LabelOptions::from(&selected_text[..]) {
            Back => {
                state.selected_option = Some(Back);
                return Ok(());
            },
            _ => {}
        }
        let selected_steam_id = selections_h_map
            .get(&selections[selection])
            .unwrap()
            .clone()
            .to_string();
        let (game_short_name, _) = find_game_conf_by_steam_id(config, &selected_steam_id)
            .change_context(DialogsErrors::GameNotFoundForGivenSteamId(
                selected_steam_id.clone(),
            ))?;
        let game_short_name = game_short_name.clone();
        let game_config = config.games.get_mut(&game_short_name);
        let conf = game_config.unwrap();
        let runtime = conf.runtime.as_ref().unwrap();
        info!(
            "Switched runtime from {:?} to {:?} for {}",
            runtime,
            runtime.as_opposite(),
            game_short_name
        );
        conf.runtime = Some(runtime.as_opposite());
        state.selected_option = Some(Back);
        Ok(())
    }
}

pub fn populate_selections_with_general_options(game_decisions: &HashMap<String, (ReleaseAsset, Option<bool>, Option<String>)>, selections: &mut Vec<String>, different_found: &bool, any_not_installed_mods_with_both_ver_supporting: &bool) -> Option<()> {
    if !game_decisions.is_empty() {
        selections.push(UpdateAllGames.to_label());
        if *different_found && !*any_not_installed_mods_with_both_ver_supporting {
            // will choose base of your current local mod settings per game
            selections[0] = UpdateAllGamesAutoDetect.to_label();
        } else if *different_found && *any_not_installed_mods_with_both_ver_supporting {
            // will choose base of your current local mod settings per game
            // for games that support both versions will choose base of below decision
            selections.push(UpdateAllGamesPreferStandard.to_label());
            selections[0] = UpdateAllGamesPreferNextgen.to_label();
        }
        return Some(());
    } else {
        return None;
    };
}

pub fn populate_selected_assets_base_on_general_option(sel: LabelOptions, game_decisions: &HashMap<String, (ReleaseAsset, Option<bool>, Option<String>)>, state: &mut REvilManagerState, different_found: &bool, any_not_installed_mods_with_both_ver_supporting: &bool) -> Option<()> {
    if sel != Skip && sel != Other {
        game_decisions.values().for_each(|(asset, include, _)| {
            if include.is_some() {
                if !include.unwrap() {
                    debug!("Asset not added {}", asset.name);
                    return;
                }
                debug!("adding asset as included true asset {}", asset.name);
                state.selected_assets.push(asset.clone());
            } else {
                if !*different_found || !*any_not_installed_mods_with_both_ver_supporting {
                    return;
                }

                if asset.name.contains(STANDARD_TYPE_QUALIFIER) {
                    if sel != UpdateAllGamesPreferStandard {
                        return;
                    }
                    debug!("adding standard asset {}", asset.name);
                    state.selected_assets.push(asset.clone())
                } else {
                    if sel != UpdateAllGamesPreferNextgen {
                        return;
                    }
                    debug!("adding nextgen asset {}", asset.name);
                    state.selected_assets.push(asset.clone());
                }
            };
        });
        return Some(());
    }
    None
}

type LabelText = String;

impl Dialogs {
    fn prepare_decision_report(
        &self,
        config: &REvilConfig,
        state: &mut REvilManagerState,
        assets_report: &AssetsReport,
    ) -> ResultManagerErr<(
        bool,
        bool,
        HashMap<LabelText, (ReleaseAsset, Option<bool>, Option<String>)>,
    )> {
        // it determines wether you have game that supports different version i.e. RE2 support both nextgen and standard but if you have only games like
        // MHRISE DMC5 then it should not change thus should not display specific message later
        let mut different_found = false;
        // it checks if any nextgen supported game doesn't have nextgen type set - treating like mod is not installed
        let mut is_any_game_support_sec_version_but_mod_is_not_installed = false;
        let mut games: HashMap<String, (ReleaseAsset, Option<bool>, Option<SteamId>)> = HashMap::new();

        assets_report.iter().for_each(|(game_short_name, assets)| {
            if !state.games_that_require_update.contains(game_short_name) {
                return;
            };
            debug!("Processing game: {}", game_short_name);

            assets.iter().for_each(|asset| {
                debug!("Processing asset: {}", asset.name);
                let game_config = config.games.get(game_short_name).unwrap();
                let (
                    text,
                    include_for_all_option,
                    mod_is_probably_not_installed,
                    does_asset_support_2_version_of_mod,
                ) = is_asset_tdb(game_short_name, asset)
                    .map(|is_tdb| {
                        const TWO_VERSION_SUPPORTED: bool = true;
                        let nextgen = match game_config.nextgen {
                            Some(it) => it,
                            None => {
                                debug!("Nextgen field is missing for {} - game supporting both version. Probably mod is not installed", game_short_name);
                                if is_tdb {
                                    return (format!("{} Standard version", game_short_name), None, true, TWO_VERSION_SUPPORTED)
                                } else {
                                    return (format!("{} Nextgen version", game_short_name), None, true, TWO_VERSION_SUPPORTED)
                                }
                            }
                        };

                        if !is_tdb {
                            let mut text = format!("{} Nextgen version", game_short_name);
                            if nextgen {
                                debug!("Asset is Nextgen like installed mod");
                                return (text, Some(true), false, TWO_VERSION_SUPPORTED);
                            };
                            debug!("Asset is TDB but installed mod is nextgen");
                            set_label_for_download_switch(&mut text, "standard", "nextgen");
                            return (text, Some(false), false, TWO_VERSION_SUPPORTED);
                        };

                        let mut text = format!("{} Standard version", game_short_name);
                        if !nextgen {
                            debug!("Asset is TDB like installed mod");
                            return (text, Some(true), false, TWO_VERSION_SUPPORTED);
                        };
                        debug!("Asset is Nextgen but installed mod is TDB");
                        set_label_for_download_switch(&mut text, "nextgen", "standard");
                        (text, Some(false), false, TWO_VERSION_SUPPORTED)
                    })
                    .unwrap_or_else(|| {
                        debug!("asset is not TDB nor Nextgen");
                         (game_short_name.to_string(), Some(true), false, false) 
                    });

                // ifs are needed because we want to assign it only for true 
                if mod_is_probably_not_installed {
                    is_any_game_support_sec_version_but_mod_is_not_installed = mod_is_probably_not_installed;
                }
                if does_asset_support_2_version_of_mod {
                    different_found = does_asset_support_2_version_of_mod;
                }
                games.insert(
                    text,
                    (
                        asset.clone(),
                        include_for_all_option,
                        game_config.steamId.clone(),
                    ),
                );
            });
        });

        Ok((
            different_found,
            is_any_game_support_sec_version_but_mod_is_not_installed,
            games,
        ))
    }
}

fn set_label_for_download_switch(text: &mut String, next_or_std: &str, next_or_std_sec: &str) {
    *text = format!(
        "{}      {}(your current version of mod is {} -> it will switch to {})",
        text,
        SORT_DETERMINER,
        next_or_std,
        next_or_std_sec
    );
}

#[cfg(test)]
mod download_decision_tests;

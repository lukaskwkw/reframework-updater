use std::{collections::HashMap, env, error::Error, path::PathBuf};

use dialoguer::{theme::ColorfulTheme, Select};
use error_stack::{Report, Result, ResultExt};
use log::{debug, info};
use self_update::update::ReleaseAsset;

use crate::{
    dialogs::dialogs_label::{LabelOptions, SWITCH_RUNTIME_PART},
    rManager::rManager::SWITCH_IDENTIFIER,
    rManager::rManager_header::{
        REvilManager, REvilManagerError, REvilManagerState, ResultManagerErr, SORT_DETERMINER,
    },
    tomlConf::configStruct::{REvilConfig, ShortGameName, SteamId},
    utils::{find_game_conf_by_steam_id::find_game_conf_by_steam_id, is_asset_tdb::is_asset_tdb},
    STANDARD_TYPE_QUALIFIER,
};

#[derive(Debug, Default)]
pub enum DialogsErrors {
    #[default]
    Other,
    GameNotFoundForGivenSteamId(String),
}
// TODO Fill above structs with more errors rather than just using Other everywhere.

impl std::fmt::Display for DialogsErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DialogsErrors")
    }
}

impl Error for DialogsErrors {}

type ResultDialogsErr<T> = Result<T, DialogsErrors>;

pub trait Ask {
    fn ask_for_decision(
        &mut self,
        config: &mut REvilConfig,
        state: &mut REvilManagerState,
        report: &HashMap<String, Vec<ReleaseAsset>>,
    ) -> ResultDialogsErr<()>;
    fn ask_for_game_decision_if_needed(
        &mut self,
        config: &mut REvilConfig,
        state: &mut REvilManagerState,
    ) -> ResultDialogsErr<()>;
    fn ask_for_switch_type_decision(
        &mut self,
        config: &mut REvilConfig,
        state: &mut REvilManagerState,
    ) -> ResultDialogsErr<SwitchActionReport>;
}

pub struct Dialogs;
impl Dialogs {
    pub fn new() -> Self {
        Self
    }
}
type SecondAssetName = String;
pub enum SwitchActionReport {
    UnzipSaveAndExit(ShortGameName, SecondAssetName),
    SaveAndRunThenExit(ShortGameName, PathBuf),
    Early,
}

impl Dialogs {
    fn prepare_decision_report(
        &self,
        config: &REvilConfig,
        state: &mut REvilManagerState,
        report: &HashMap<String, Vec<ReleaseAsset>>,
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

        // TODO next-time be careful with those combinators
        report.iter().for_each(|(game_short_name, assets)| {
            state
                .games_that_require_update
                .contains(game_short_name)
                .then(|| {
                    assets.iter().for_each(|asset| {
                        let mut text = "".to_string();
                        let mut include_for_all_option = Some(true);
                        let game_config = config.games.get(game_short_name).unwrap();
                        is_asset_tdb(game_short_name, asset)
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
                                    if nextgen.is_none() {
                                        debug!("None for {}, {}", game_short_name, asset.name);
                                        include_for_all_option = None;
                                        any_none = true;
                                    }
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

impl Ask for Dialogs {
    fn ask_for_decision(
        &mut self,
        config: &mut REvilConfig,
        state: &mut REvilManagerState,
        report: &HashMap<String, Vec<ReleaseAsset>>,
    ) -> ResultDialogsErr<()> {
        let (different_found, any_none, games) = &self
            .prepare_decision_report(config, state, report)
            .change_context(DialogsErrors::Other)?;
        let mut selections = vec![];
        use LabelOptions::*;
        if games.len() > 0 {
            selections.push(UpdateAllGames.to_label());
            // TODO when some game has nextgen as false it thins like it is none I've added sterix next to any_none does it fix it ?
            if *different_found && !*any_none {
                // will choose base of your current local mod settings per game
                selections[0] = UpdateAllGamesAutoDetect.to_label();
            } else if *different_found && *any_none {
                selections.push(UpdateAllGamesPreferStandard.to_label());
                selections[0] = UpdateAllGamesPreferNextgen.to_label();
            }
        } else {
            info!("Not found any games to update");
            return Ok(());
        };

        let mut texts: Vec<String> = games.keys().cloned().collect();
        texts.sort_by(|a, b| REvilManager::sort(a, b));
        selections.extend(texts);
        debug!("{:#?}", selections);

        let count = state.games_that_require_update.len();
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
                return Ok(());
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
                        state.selected_assets.push(asset.clone());
                    })
                    .unwrap_or_else(|| {
                        (*different_found && *any_none).then(|| {
                            if asset.name.contains(STANDARD_TYPE_QUALIFIER) {
                                if sel != UpdateAllGamesPreferStandard {
                                    return;
                                }
                                debug!("adding standard asset for {}", asset.name);
                                state.selected_assets.push(asset.clone())
                            } else {
                                if sel != UpdateAllGamesPreferNextgen {
                                    return;
                                }
                                debug!("adding nextgen asset for {}", asset.name);
                                state.selected_assets.push(asset.clone());
                            }
                        });
                    });
            });
            return Ok(());
        }

        if let Some((asset, _, game_id)) = games.get(&selections[selection]) {
            debug!("adding asset {}", asset.name);
            state.selected_assets.push(asset.clone().clone());
            state.selected_game_to_launch = game_id.clone();
        };
        Ok(())
    }

    fn ask_for_game_decision_if_needed(
        &mut self,
        config: &mut REvilConfig,
        state: &mut REvilManagerState,
    ) -> ResultDialogsErr<()> {
        if state.selected_game_to_launch.is_some() {
            return Ok(());
        }

        let mut selections_h_map: HashMap<String, &SteamId> = HashMap::new();
        // let conf = config;
        config.games.iter().for_each(|(short_name, game)| {
            game.versions
                .as_ref()
                .and_then(|versions| {
                    (versions.first().unwrap().len() > 1 && game.runtime.is_some()).then(|| {
                        selections_h_map.insert(
                            format!(
                                "{}: {} <{:?}> for {} and run game",
                                SORT_DETERMINER,
                                SWITCH_RUNTIME_PART,
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
                state.selected_option = Some(Exit);
                return Ok(());
            }
            SwitchType => {
                state.selected_option = Some(SwitchType);
                return Ok(());
            }
            _ => (),
        };

        let selected_steam_id = selections_h_map
            .get(&selections[selection])
            .unwrap()
            .clone()
            .to_string();

        if selected_text.contains(SORT_DETERMINER) {
            let (game_short_name, _) = find_game_conf_by_steam_id(config, &selected_steam_id)
                .change_context(DialogsErrors::GameNotFoundForGivenSteamId(
                    selected_steam_id.clone(),
                ))?;
            let game_short_name = game_short_name.clone();
            let game_config = config.games.get_mut(&game_short_name);
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
        state.selected_game_to_launch = Some(selected_steam_id.clone().to_string());
        Ok(())
    }

    fn ask_for_switch_type_decision(
        &mut self,
        config: &mut REvilConfig,
        state: &mut REvilManagerState,
    ) -> ResultDialogsErr<SwitchActionReport> {
        let selected_option = state.selected_option.as_ref();
        use LabelOptions::*;
        use SwitchActionReport::*;
        if selected_option.is_none()
            || selected_option.is_some() && selected_option.unwrap() != &SwitchType
        {
            return Ok(Early);
        }
        let mut selections: Vec<String> = Vec::new();

        config.games.iter().for_each(|(short_name, game)| {
            game.nextgen
                .map(|nextgen| {
                    if nextgen {
                        selections.push(SwitchToStandard(short_name.to_string()).to_label());
                    } else {
                        selections.push(SwitchToNextgen(short_name.to_string()).to_label());
                    }
                })
                .unwrap_or_default();
        });
        selections.push(Exit.to_label());
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Select game to switch"))
            .default(0)
            .items(&selections[..])
            .interact()
            .unwrap();
        let selected_text = &selections[selection];

        match LabelOptions::from(&selected_text[..]) {
            Exit => {
                info!("Chosen exit option. Bye bye..");
                return Ok(Early);
            }
            SwitchToStandard(short_name) | SwitchToNextgen(short_name) => {
                debug!("Selected -> {:#?}", short_name);
                let game_config = config.games.get_mut(&short_name).unwrap();
                let nextgen = game_config.nextgen.as_ref().unwrap();
                game_config.nextgen = Some(!nextgen);

                if !state.games_that_require_update.contains(&short_name) {
                    let next_gen = game_config.nextgen.unwrap();
                    let versions = game_config.versions.as_mut().unwrap();
                    let first_set = versions.first_mut().unwrap();

                    let second_asset_name = first_set
                        .iter()
                        .skip(1)
                        .find(|name| {
                            is_asset_tdb(
                                &short_name,
                                &ReleaseAsset {
                                    name: name.to_string(),
                                    ..Default::default()
                                },
                            )
                            .map(|is_tdb| ((is_tdb && !next_gen) || (!is_tdb && next_gen)))
                            .unwrap_or_default()
                        })
                        .unwrap_or(&"dupa".to_string())
                        .clone();

                    // I didn't know how to solve borrow checker issue so if there is no second cache asset then it's called "dupa"
                    if second_asset_name != "dupa" {
                        debug!("preparing unzip for {}", second_asset_name);
                        // TODO if asset from cache is missing then it will panic maybe make it to download asset instead?
                        // but it requires first removing it from versions, saving config and running process again
                        // TODO change it to either launch the game or back to previous section after ok
                        return Ok(UnzipSaveAndExit(short_name, second_asset_name));
                    }

                    let version = first_set.first_mut().unwrap();
                    *version = SWITCH_IDENTIFIER.to_string();
                } else {
                    debug!("Game {} requires update anyway", short_name);
                }
                let path = env::current_exe()
                    .map(|path| path)
                    .or_else(|err| {
                        Err(Report::new(REvilManagerError::Other)
                            .attach_printable(format!("current_exe {}", err)))
                    })
                    .change_context(DialogsErrors::Other)?;
                return Ok(SaveAndRunThenExit(short_name, path));
            }
            _ => (),
        };
        Ok(Early)
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

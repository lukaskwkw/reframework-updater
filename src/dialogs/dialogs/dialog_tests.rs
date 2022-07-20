use std::collections::HashMap;

use log::debug;
use self_update::update::ReleaseAsset;

use super::Dialogs;
use crate::{
    dialogs::{
        dialogs::{
            populate_selected_assets_base_on_general_option,
            populate_selections_with_general_options,
        },
        dialogs_label::LabelOptions,
    },
    rManager::rManager_header::REvilManagerState,
    tests::{
        config_provider_mock::mock_conf_provider::get_config_provider_mock,
        refr_github_mock::{prepare_refr_github_mock_and_get_constr, MockManageGithubM},
    },
    tomlConf::{config::ConfigProvider, configStruct::REvilConfig},
    utils::init_logger::init_logger,
};
use std::sync::Once;

static INIT: Once = Once::new();
fn init() -> (
    HashMap<String, Vec<ReleaseAsset>>,
    REvilConfig,
    REvilManagerState,
    Box<Dialogs>,
) {
    INIT.call_once(|| {
        // init_logger("debug"); // uncomment if need more data for debugging
    });

    let (ctx, refr_constr) = prepare_refr_github_mock_and_get_constr();
    let refr_github = refr_constr("something", "anything");
    let assets_report = refr_github.getAssetsReport().clone();
    let config_provider = get_config_provider_mock() as Box<dyn ConfigProvider>;
    let config = config_provider.load_from_file().unwrap();
    let state: REvilManagerState = REvilManagerState::default();
    let dialogs = Box::new(Dialogs);
    (assets_report, config, state, dialogs)
}

use LabelOptions::*;

#[test]
fn for_4_games() {
    println!("Given 4 games to update, all different types of, should pass assertions");
    let (assets_report, config, mut state, dialogs) = init();

    state.games_that_require_update.push("RE8".to_string());
    state.games_that_require_update.push("RE7".to_string());
    state.games_that_require_update.push("RE2".to_string());
    state.games_that_require_update.push("RE3".to_string());

    let (different_found, any_not_installed_mods_with_both_ver_supporting, game_decisions) =
        dialogs
            .prepare_decision_report(&config, &mut state, &assets_report)
            .unwrap();

    assert!(different_found);
    assert!(any_not_installed_mods_with_both_ver_supporting);
    assert_eq!(game_decisions.len(), 7);

    {
        let (asset, include, _) = game_decisions.get("RE8").unwrap();
        assert!(include.unwrap());
        assert_eq!(asset.name, "RE8.zip".to_string());
    }

    {
        let (asset, include, _) = game_decisions.get("RE7 Standard version").unwrap();
        assert!(include.is_none());
        assert_eq!(asset.name, "RE7_TDBXXX.zip".to_string());
    }

    {
        let (asset, include, _) = game_decisions.get("RE7 Nextgen version").unwrap();
        assert!(include.is_none());
        assert_eq!(asset.name, "RE7.zip".to_string());
    }

    {
        let (asset, include, _) = game_decisions.get("RE3 Standard version").unwrap();
        assert!(include.unwrap());
        assert_eq!(asset.name, "RE3_TDBXXX.zip".to_string());
    }

    let (asset, include, _) = game_decisions
        .iter()
        .find(|(key, _)| key.contains("RE2 Standard"))
        .unwrap()
        .1;
    assert_eq!(asset.name, "RE2_TDBXXX.zip".to_string());
    assert!(!include.unwrap());

    let (asset, include, _) = game_decisions
        .iter()
        .find(|(key, _)| key.contains("RE3 Nextgen"))
        .unwrap()
        .1;
    assert_eq!(asset.name, "RE3.zip".to_string());
    assert!(!include.unwrap());

    let (asset, include, _) = game_decisions
        .iter()
        .find(|(key, _)| key.contains("RE2 Nextgen"))
        .unwrap()
        .1;

    assert_eq!(asset.name, "RE2.zip".to_string());
    assert!(include.unwrap());

    let mut selections: Vec<String> = Vec::new();
    populate_selections_with_general_options(
        &game_decisions,
        &mut selections,
        &different_found,
        &any_not_installed_mods_with_both_ver_supporting,
    )
    .unwrap();

    // Should return 2 options as mod for RE7 is not installed but supports both versions
    assert_eq!(selections.len(), 2);
    assert_eq!(selections[0], UpdateAllGamesPreferNextgen.to_label());
    assert_eq!(selections[1], UpdateAllGamesPreferStandard.to_label());
}

#[test]
fn for_4_games_and_chosen_prefer_standard_decision() {
    println!("Given 4 games to update, different types of, and chosen standard decision - should pass assertions");
    let (assets_report, config, mut state, dialogs) = init();

    state.games_that_require_update.push("RE8".to_string());
    state.games_that_require_update.push("RE7".to_string());
    state.games_that_require_update.push("RE2".to_string());
    state.games_that_require_update.push("RE3".to_string());

    let (different_found, any_not_installed_mods_with_both_ver_supporting, game_decisions) =
        dialogs
            .prepare_decision_report(&config, &mut state, &assets_report)
            .unwrap();

    populate_selected_assets_base_on_general_option(
        UpdateAllGamesPreferStandard,
        &game_decisions,
        &mut state,
        &different_found,
        &any_not_installed_mods_with_both_ver_supporting,
    )
    .unwrap();
    let expected_results = [
        "RE3_TDBXXX.zip".to_string(),
        "RE2.zip".to_string(),
        "RE7_TDBXXX.zip".to_string(),
        "RE8.zip".to_string(),
    ]
    .to_vec();

    state.selected_assets.iter().for_each(|asset| {
        let found = expected_results.iter().find(|name| **name == asset.name);
        assert!(found.is_some());
    });
}
#[test]
fn for_4_games_and_chosen_prefer_nextgen_decision() {
    println!("Given 4 games to update, different types of, and chosen nextgen decision - should pass assertions");
    let (assets_report, config, mut state, dialogs) = init();

    state.games_that_require_update.push("RE8".to_string());
    state.games_that_require_update.push("RE7".to_string());
    state.games_that_require_update.push("RE2".to_string());
    state.games_that_require_update.push("RE3".to_string());

    let (different_found, any_not_installed_mods_with_both_ver_supporting, game_decisions) =
        dialogs
            .prepare_decision_report(&config, &mut state, &assets_report)
            .unwrap();

    populate_selected_assets_base_on_general_option(
        UpdateAllGamesPreferNextgen,
        &game_decisions,
        &mut state,
        &different_found,
        &any_not_installed_mods_with_both_ver_supporting,
    )
    .unwrap();
    let expected_results = [
        "RE7.zip".to_string(),
        "RE3_TDBXXX.zip".to_string(),
        "RE2.zip".to_string(),
        "RE8.zip".to_string(),
    ]
    .to_vec();

    state.selected_assets.iter().for_each(|asset| {
        let found = expected_results.iter().find(|name| **name == asset.name);
        assert!(found.is_some());
    });
}

#[test]
fn for_2_games() {
    println!("Given 2 games to update, one can support nextgen second not");
    let (assets_report, config, mut state, dialogs) = init();

    state.games_that_require_update.push("RE2".to_string());
    state.games_that_require_update.push("RE8".to_string());

    let (different_found, any_not_installed_mods_with_both_ver_supporting, game_decisions) =
        dialogs
            .prepare_decision_report(&config, &mut state, &assets_report)
            .unwrap();
    assert!(different_found);
    assert!(!any_not_installed_mods_with_both_ver_supporting);
    assert_eq!(game_decisions.len(), 3);
    let mut selections: Vec<String> = Vec::new();
    populate_selections_with_general_options(
        &game_decisions,
        &mut selections,
        &different_found,
        &any_not_installed_mods_with_both_ver_supporting,
    )
    .unwrap();
    assert_eq!(selections.len(), 1);
    assert_eq!(selections[0], UpdateAllGamesAutoDetect.to_label());
}

#[test]
fn for_1_game() {
    println!("Given 1 game to update not supporting nextgen version");
    let (assets_report, config, mut state, dialogs) = init();

    state.games_that_require_update.push("RE8".to_string());

    let (different_found, any_not_installed_mods_with_both_ver_supporting, game_decisions) =
        dialogs
            .prepare_decision_report(&config, &mut state, &assets_report)
            .unwrap();
    assert!(!different_found);
    assert!(!any_not_installed_mods_with_both_ver_supporting);
    assert_eq!(game_decisions.len(), 1);
    let mut selections: Vec<String> = Vec::new();
    populate_selections_with_general_options(
        &game_decisions,
        &mut selections,
        &different_found,
        &any_not_installed_mods_with_both_ver_supporting,
    )
    .unwrap();
    assert_eq!(selections.len(), 1);
    assert_eq!(selections[0], UpdateAllGames.to_label());
}

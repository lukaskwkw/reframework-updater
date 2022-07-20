use std::collections::HashMap;

use log::debug;
use self_update::update::ReleaseAsset;

use crate::{
    rManager::rManager_header::REvilManagerState,
    tests::{
        config_provider_mock::mock_conf_provider::get_config_provider_mock,
        refr_github_mock::{prepare_refr_github_mock_and_get_constr, MockManageGithubM},
    },
    tomlConf::{config::ConfigProvider, configStruct::REvilConfig},
    utils::init_logger::init_logger,
};
use super::Dialogs;
use std::sync::Once;

static INIT: Once = Once::new();
fn init() -> (HashMap<String, Vec<ReleaseAsset>>, REvilConfig, REvilManagerState, Box<Dialogs>) {
    INIT.call_once(|| {
        init_logger("debug");
    });

    let (ctx, refr_constr) = prepare_refr_github_mock_and_get_constr();
    let REFR_Github = refr_constr("something", "balbal");
    let assets_report = REFR_Github.getAssetsReport().clone();
    let mut config_provider = get_config_provider_mock() as Box<dyn ConfigProvider>;
    let config = config_provider.load_from_file().unwrap();
    let mut state: REvilManagerState = REvilManagerState::default();
    let dialogs = Box::new(Dialogs);
    (assets_report, config, state, dialogs)
}


#[test]
fn for_3_games() {
    println!("Given 3 games to update, different types of, should pass assertions");
    let (assets_report, config, mut state, dialogs) = init();
    
    state.games_that_require_update.push("RE8".to_string());
    state.games_that_require_update.push("RE7".to_string());
    state.games_that_require_update.push("RE2".to_string());

    let (different_found, any_not_installed_mods_with_both_ver_supporting, game_decisions) = dialogs.prepare_decision_report(&config, &mut state, &assets_report).unwrap();
    assert!(different_found);
    assert!(any_not_installed_mods_with_both_ver_supporting);
    assert_eq!(game_decisions.len(), 4);
}

#[test]
fn for_2_games() {
    println!("Given 2 games to update, one can support nextgen second not");
    let (assets_report, config, mut state, dialogs) = init();
    
    state.games_that_require_update.push("RE8".to_string());
    state.games_that_require_update.push("RE2".to_string());

    let (different_found, any_not_installed_mods_with_both_ver_supporting, game_decisions) = dialogs.prepare_decision_report(&config, &mut state, &assets_report).unwrap();
    assert!(different_found);
    assert!(!any_not_installed_mods_with_both_ver_supporting);
    assert_eq!(game_decisions.len(), 3);
}

#[test]
fn for_1_game() {
    println!("Given 1 game to update not supporting nextgen version");
    let (assets_report, config, mut state, dialogs) = init();
    
    state.games_that_require_update.push("RE8".to_string());

    let (different_found, any_not_installed_mods_with_both_ver_supporting, game_decisions) = dialogs.prepare_decision_report(&config, &mut state, &assets_report).unwrap();
    assert!(!different_found);
    assert!(!any_not_installed_mods_with_both_ver_supporting);
    assert_eq!(game_decisions.len(), 1);
}

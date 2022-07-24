#[cfg(test)]
pub mod tests {
    use error_stack::Report;

    use crate::args::{ArgsClap, RunAfter};
    use crate::dialogs::dialogs::{Ask, MockAsk};
    use crate::strategy::StrategyFactory::StrategyFactory;
    use crate::tests::config_provider_mock::mock_conf_provider::load_from_file_default_return_mock;

    use crate::rManager::rManager_header::REvilManager;
    use crate::tests::dialog_provider_mock::{
        ask_for_decision_and_populate_selected_assets_return_mock,
        ask_for_game_decision_if_needed_return_mock,
    };
    use crate::tests::manager_mocks::init_manager_mocks;
    use crate::tomlConf::configStruct::{ConfigError, ErrorLevel};
    use crate::utils::local_version::LocalGameConfig;
    use crate::ARGS;

    #[test]
    fn default_route() {
        let games_steam_id = ["883710", "418370", "1196590", "952060"].to_vec();
        games_steam_id.iter().for_each(|steam_id| {
            unsafe {
                ARGS = Some(ArgsClap {
                    level: ErrorLevel::info,
                    one: "none".to_string(),
                    run: RunAfter::no,
                });
            }
            let (
                mut steam_menago,
                local_provider_mock,
                mut dialogs,
                mut config_provider_mock,
                ctx,
                mock_reft_constr,
            ) = init_manager_mocks();
            let id = steam_id.clone();

            dialogs
                .expect_ask_for_decision_and_populate_selected_assets()
                .returning(ask_for_decision_and_populate_selected_assets_return_mock());

            dialogs
                .expect_ask_for_game_decision_if_needed_and_set_game_to_launch()
                .returning(ask_for_game_decision_if_needed_return_mock(id.to_string()));

            steam_menago
                .expect_run_game_via_steam_manager()
                .withf(|x| {
                    assert_eq!(x.to_string(), id.to_string()); // added assert_eq! to have better error message without it is not obvious what really happened
                    true
                })
                .once()
                .returning(|_| Ok(()));

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

            let strategy = StrategyFactory::get_strategy(&mut evil_manager);
            strategy(&mut evil_manager);

            // for RE7 RE2 RE8 should alter configs after download but for RE3 config should stay the same

            let game = evil_manager.config.games.get("RE2").unwrap();
            assert!(!game.nextgen.unwrap());
            assert_eq!(
                game.version_in_use.clone().unwrap(),
                "v1.333-07ab146".to_string()
            );
            // below test if old location has been altered to new location
            let game = evil_manager.config.games.get("RE2").unwrap();
            assert_eq!(
                game.location.clone().unwrap(),
                "D:/steam/games/RE2".to_string()
            );

            let game = evil_manager.config.games.get("RE3").unwrap();
            assert!(!game.nextgen.unwrap());
            assert_eq!(
                game.version_in_use.clone().unwrap(),
                "v1.71-abd3145".to_string()
            );

            let game = evil_manager.config.games.get("RE8").unwrap();
            assert!(game.nextgen.is_none());
            // steamId is missing in config but should be altered after steam detection
            assert_eq!(game.steamId, Some("1196590".to_string()));

            let game = evil_manager.config.games.get("RE7").unwrap();
            assert!(!game.nextgen.unwrap());

        });
    }

    #[test]
    fn default_route_but_load_from_file_failed() {
        let games_steam_id = ["883710", "418370", "1196590", "952060"].to_vec();
        games_steam_id.iter().for_each(|steam_id| {
            unsafe {
                ARGS = Some(ArgsClap {
                    level: ErrorLevel::info,
                    one: "none".to_string(),
                    run: RunAfter::no,
                });
            }
            let (
                mut steam_menago,
                mut local_provider_mock,
                mut dialogs,
                mut config_provider_mock,
                ctx,
                mock_reft_constr,
            ) = init_manager_mocks();
            let id = steam_id.clone();

            if *steam_id != "952060" {
                steam_menago
                    .expect_run_game_via_steam_manager()
                    .withf(|x| {
                        assert_eq!(x.to_string(), id.to_string());
                        true
                    })
                    .once()
                    .returning(|_| Ok(()));
            } else {
                steam_menago
                    .expect_run_game_via_steam_manager()
                    .never()
                    .returning(|_| Ok(()));
            }

            config_provider_mock
                .expect_load_from_file()
                .returning(|| Err(Report::new(ConfigError::ConfigFileError)));

            local_provider_mock
                .expect_get_local_report_for_game()
                .returning(|_, _| LocalGameConfig::default());

            dialogs
                .expect_ask_for_decision_and_populate_selected_assets()
                .returning(ask_for_decision_and_populate_selected_assets_return_mock());
            dialogs
                .expect_ask_for_game_decision_if_needed_and_set_game_to_launch()
                .returning(ask_for_game_decision_if_needed_return_mock(id.to_string()));

            let mut evil_manager = REvilManager::new(
                config_provider_mock,
                local_provider_mock,
                steam_menago,
                dialogs,
                mock_reft_constr,
            );

            let strategy = StrategyFactory::get_strategy(&mut evil_manager);
            strategy(&mut evil_manager);

            // for RE7 RE2 RE8 should alter configs after download but for RE3 config should stay the same

            let game = evil_manager.config.games.get("RE2").unwrap();
            assert!(!game.nextgen.unwrap());
            assert_eq!(
                game.version_in_use.clone().unwrap(),
                "v1.333-07ab146".to_string()
            );

            let game = evil_manager.config.games.get("RE2").unwrap();
            assert_eq!(
                game.location.clone().unwrap(),
                "D:/steam/games/RE2".to_string()
            );

            let game = evil_manager.config.games.get("RE3").unwrap();
            assert!(game.nextgen.is_none());
            assert!(game.location.is_some());
            assert!(game.version_in_use.is_none());

            let game = evil_manager.config.games.get("RE8").unwrap();
            assert!(game.nextgen.is_none());

            let game = evil_manager.config.games.get("RE7").unwrap();
            assert!(!game.nextgen.unwrap());
        });
    }

    #[test]
    fn check_update_and_run_the_game_route() {
        let games = ["RE2", "RE3", "RE7", "RE8"].to_vec();
        games.iter().for_each(|short_name| {
            let (
                mut steam_menago,
                local_provider_mock,
                _,
                mut config_provider_mock,
                ctx,
                mock_reft_constr,
            ) = init_manager_mocks();

            config_provider_mock
                .expect_load_from_file()
                .returning(load_from_file_default_return_mock());

            let mock_ask = MockAsk::new();

            let mut dialogs = Box::new(mock_ask);
            dialogs
                .expect_ask_for_decision_and_populate_selected_assets()
                .never();

            dialogs
                .expect_ask_for_game_decision_if_needed_and_set_game_to_launch()
                .returning(|_, state| Ok(()));

            steam_menago
                .expect_run_game_via_steam_manager()
                .once()
                .returning(|_| Ok(()));

            let mut evil_manager = REvilManager::new(
                config_provider_mock,
                local_provider_mock,
                steam_menago,
                dialogs,
                mock_reft_constr,
            );

            unsafe {
                ARGS = Some(ArgsClap {
                    level: ErrorLevel::info,
                    one: short_name.to_string(),
                    run: RunAfter::yes,
                });
            }
            let strategy = StrategyFactory::get_strategy(&mut evil_manager);
            strategy(&mut evil_manager);
        })

    }

    #[test]
    fn check_update_and_run_the_game_route_but_load_from_file_failed() {
        let games = ["RE2", "RE3", "RE7", "RE8"].to_vec();
        games.iter().for_each(|short_name| {
            let (
                mut steam_menago,
                mut local_provider_mock,
                _,
                mut config_provider_mock,
                ctx,
                mock_reft_constr,
            ) = init_manager_mocks();

            config_provider_mock
                .expect_load_from_file()
                .returning(|| Err(Report::new(ConfigError::ConfigFileError)));

            steam_menago
                .expect_run_game_via_steam_manager()
                .once()
                .returning(|_| Ok(()));
            let mock_ask = MockAsk::new();

            let mut dialogs = Box::new(mock_ask);
            dialogs
                .expect_ask_for_decision_and_populate_selected_assets()
                .never();

            dialogs
                .expect_ask_for_game_decision_if_needed_and_set_game_to_launch()
                .returning(|_, state| Ok(()));

            let mut evil_manager = REvilManager::new(
                config_provider_mock,
                local_provider_mock,
                steam_menago,
                dialogs,
                mock_reft_constr,
            );

            unsafe {
                ARGS = Some(ArgsClap {
                    level: ErrorLevel::info,
                    one: short_name.to_string(),
                    run: RunAfter::yes,
                });
            }
            let strategy = StrategyFactory::get_strategy(&mut evil_manager);
            strategy(&mut evil_manager);
        })

    }
}

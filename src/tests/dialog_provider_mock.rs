use self_update::update::ReleaseAsset;

use crate::dialogs::dialogs::SwitchActionReport;

use crate::dialogs::dialogs::MockAsk;

pub fn get_dialogs_provider_mock() -> Box<MockAsk> {
    let mock_ask = MockAsk::new();

    let mut dialogs = Box::new(mock_ask);
    dialogs
        .expect_ask_for_decision_and_populate_selected_assets()
        .returning(|config, state, report| {
            state.selected_assets.push(ReleaseAsset {
                download_url: "url".to_string(),
                name: "RE7_TDBXXX.zip".to_string(),
            });
            state.selected_assets.push(ReleaseAsset {
                download_url: "url".to_string(),
                name: "RE2.zip".to_string(),
            });
            state.selected_assets.push(ReleaseAsset {
                download_url: "url".to_string(),
                name: "RE8.zip".to_string(),
            });
            Ok(())
        });
    dialogs
        .expect_ask_for_game_decision_if_needed_and_set_game_to_launch()
        .returning(|_, state| {
            state.selected_game_to_launch = Some("418370".to_string());
            Ok(())
        });
    dialogs
        .expect_get_switch_type_decision()
        .returning(|_, _| Ok(SwitchActionReport::Early));

    dialogs
}

use std::collections::HashMap;

use self_update::update::ReleaseAsset;

use crate::dialogs::dialogs::DialogsErrors;
use crate::dialogs::dialogs::ResultDialogsErr;
use crate::dialogs::dialogs::SwitchActionReport;

use crate::dialogs::dialogs::MockAsk;
use crate::rManager::rManager_header::REvilManagerState;
use crate::tomlConf::configStruct::REvilConfig;

pub fn get_dialogs_provider_mock() -> Box<MockAsk> {
    let mock_ask = MockAsk::new();

    let mut dialogs = Box::new(mock_ask);
    dialogs
        .expect_get_switch_type_decision()
        .returning(|_, _| Ok(SwitchActionReport::Early));

    dialogs
}

pub fn ask_for_game_decision_if_needed_return_mock(
    steam_id: String,
) -> Box<dyn Fn(&mut REvilConfig, &mut REvilManagerState) -> ResultDialogsErr<()> + Send> {
    let default = move |_: &mut REvilConfig, state: &mut REvilManagerState| {
        state.selected_game_to_launch = Some(steam_id.clone());
        Ok(())
    };
    return Box::new(default);
}

pub fn ask_for_decision_and_populate_selected_assets_return_mock() -> Box<
    dyn Fn(
            &mut REvilConfig,
            &mut REvilManagerState,
            &HashMap<String, Vec<ReleaseAsset>>,
        ) -> ResultDialogsErr<()>
        + Send,
> {
    let default = move |_: &mut REvilConfig,
                        state: &mut REvilManagerState,
                        _: &HashMap<String, Vec<ReleaseAsset>>| {
        state.selected_assets.push(ReleaseAsset {
            download_url: "url".to_string(),
            name: "RE7_TDBXXX.zip".to_string(),
        });
        state.selected_assets.push(ReleaseAsset {
            download_url: "url".to_string(),
            name: "RE2_TDBXXX.zip".to_string(),
        });
        state.selected_assets.push(ReleaseAsset {
            download_url: "url".to_string(),
            name: "RE8.zip".to_string(),
        });
        Ok(())
    };
    return Box::new(default);
}

use crate::dialogs::dialogs::SwitchActionReport;

use crate::dialogs::dialogs::MockAsk;

pub fn get_dialogs_provider_mock() -> Box<MockAsk> {
    let mock_ask = MockAsk::new();

    let mut dialogs = Box::new(mock_ask);
    dialogs
        .expect_ask_for_decision_and_populate_selected_assets()
        // TODO populate selected assets below!
        .returning(|_, _, _| Ok(()));
    dialogs
        // TODO add game to launch
        .expect_ask_for_game_decision_if_needed_and_set_game_to_launch()
        .returning(|_, _| Ok(()));
    dialogs
        .expect_get_switch_type_decision()
        .returning(|_, _| Ok(SwitchActionReport::Early));

    dialogs
}

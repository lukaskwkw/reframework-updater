use crate::tomlConf::config::ConfigProvider;

use crate::tests::config_provider_mock::mock_conf_provider::{
    get_config_provider_mock, load_from_file_default_return_mock,
};

use crate::tests::refr_github_mock::prepare_refr_github_mock_and_get_constr;


use super::super::Dialogs;

use crate::rManager::rManager_header::REvilManagerState;

use crate::tomlConf::configStruct::REvilConfig;

use self_update::update::ReleaseAsset;

use std::collections::HashMap;

use std::sync::Once;

pub(crate) static INIT: Once = Once::new();

pub(crate) fn init_dialogs_mocks() -> (
    HashMap<String, Vec<ReleaseAsset>>,
    REvilConfig,
    REvilManagerState,
    Box<Dialogs>,
) {
    INIT.call_once(|| {
        // init_logger("debug"); // uncomment if need more data for debugging
    });

    let (_ctx, refr_constr) = prepare_refr_github_mock_and_get_constr();
    let refr_github = refr_constr("something", "anything");
    let assets_report = refr_github.getAssetsReport().clone();
    let mut config_provider_mock = get_config_provider_mock();
    config_provider_mock
        .expect_load_from_file()
        .returning(load_from_file_default_return_mock());
    let config = config_provider_mock.load_from_file().unwrap();
    let state: REvilManagerState = REvilManagerState::default();
    let dialogs = Box::new(Dialogs);
    (assets_report, config, state, dialogs)
}

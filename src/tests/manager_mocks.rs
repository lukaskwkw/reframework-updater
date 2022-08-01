use crate::dialogs::dialogs::MockAsk;
use crate::reframework_github::refr_github::{ManageGithub, REFRGithub, __mock_MockREFRGithub};
use crate::tests::config_provider_mock::mock_conf_provider::get_config_provider_mock;

use crate::tests::local_provider_mock::mock_local_provider_w_defaults;
use crate::tests::refr_github_mock::prepare_refr_github_mock_and_get_constr;

use crate::tests::dialog_provider_mock::get_dialogs_provider_mock;

use crate::tests::steam_mock::prepare_steam_mock;

use crate::utils::local_version::MockLocalFiles;

use crate::steam::MockSteamThings;

pub(crate) fn init_manager_mocks() -> (
    Box<MockSteamThings>,
    Box<MockLocalFiles>,
    Box<MockAsk>,
    Box<crate::tomlConf::config::MockConfigProvider>,
    __mock_MockREFRGithub::__new::Context,
    fn(&str, &str) -> Box<dyn ManageGithub<REFRGithub>>,
) {
    let mock_steam_things = MockSteamThings::new();
    let mock_local_files = MockLocalFiles::new();
    let mut steam_menago = Box::new(mock_steam_things);
    let mut local_provider_mock = Box::new(mock_local_files);
    prepare_steam_mock(&mut steam_menago);
    mock_local_provider_w_defaults(&mut local_provider_mock);

    let dialogs = get_dialogs_provider_mock();
    // ctx variable has to be present even if it's not used - don't know why
    let config_provider_mock = get_config_provider_mock();
    let (ctx, mock_reft_constr) = prepare_refr_github_mock_and_get_constr();

    (
        steam_menago,
        local_provider_mock,
        dialogs,
        config_provider_mock,
        ctx,
        mock_reft_constr,
    )
}

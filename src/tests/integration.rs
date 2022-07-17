#[cfg(test)]
pub mod tests {
    use std::ffi::OsStr;

    use crate::args::{ArgsClap, RunAfter};
    use crate::strategy::StrategyFactory::StrategyFactory;
    use crate::tests::config_provider_mock::mock_conf_provider::get_config_provider_mock;
    use crate::tests::dialog_provider_mock::get_dialogs_provider_mock;
    use crate::tests::refr_github_mock::prepare_refr_github_mock_and_get_constr;
    use crate::tests::steam_mock::prepare_steam_mock;
    use crate::tomlConf::configStruct::ErrorLevel;
    use crate::ARGS;
    use crate::{
        rManager::rManager_header::REvilManager, steam::MockSteamThings,
        utils::local_version::MockLocalFiles,
    };

    // TODO maybe finish this below
    // or maybe it wont be necessary as not going to test other modules than rManager
    // just check for .exists fs:: and similar in rManager 
    pub struct Path {}
    impl Path {
        pub fn new(path: &str) -> Self {
            Self { }
        }
        fn exists() -> bool {
            true
        }
    }

    #[test]
    fn default_route() {
        unsafe {
            ARGS = Some(ArgsClap {
                level: ErrorLevel::debug,
                one: "none".to_string(),
                run: RunAfter::no,
            });
        }
        let mock_steam_things = MockSteamThings::new();
        let mock_local_files = MockLocalFiles::new();

        let mut steam_menago = Box::new(mock_steam_things);
        let mut local_provider = Box::new(mock_local_files);
        prepare_steam_mock(&mut steam_menago);
        local_provider
            .expect_create_ms_lnk()
            .returning(|_, _, _| Ok(()));

        let dialogs = get_dialogs_provider_mock();

        // local_provider.
        // ctx variable has to be present even if it's not used - don't know why
        let (ctx, mock_reft_constr) = prepare_refr_github_mock_and_get_constr();

        let mut evil_manager = REvilManager::new(
            get_config_provider_mock(),
            local_provider,
            steam_menago,
            dialogs,
            mock_reft_constr,
        );

        let strategy = StrategyFactory::get_strategy(&mut evil_manager);
        strategy(&mut evil_manager);

        println!("{:#?}", evil_manager.state.selected_assets);
    }
}

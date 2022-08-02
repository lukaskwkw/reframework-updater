#[cfg(test)]
pub mod mock_conf_provider {
    use crate::tomlConf::{
        config::{ConfigResult, MockConfigProvider},
        configStruct::REvilConfig,
        utils::deserialize,
    };

    pub fn get_config_provider_mock() -> Box<MockConfigProvider> {
        let mock_config_provider = MockConfigProvider::new();

        let mut config_provider = Box::new(mock_config_provider);

        // TODO this should be called once but not sure how to make it compatible with dialogs_tests so
        // it is returning instead return_once
        config_provider.expect_save_to_file().returning(|_| Ok(()));
        config_provider
    }

    pub fn load_from_file_default_return_mock() -> Box<dyn Fn() -> ConfigResult<REvilConfig> + Send>
    {
        let default = || -> ConfigResult<REvilConfig> {
            let content = r#"
        [main]
        sources = ["normal", "nightly"]
        autoupdate = true
        steamExePath = "C:\\Program Files (x86)\\Steam\\steam.exe"
    
        [RE2]
        location = "D:/steam/old_location_games/RE2"
        version_in_use = "v1.70-rbd3145"
        versions = [["v1.71-abd3145"],["v1.70-rbd3145", "RE2.zip"],["v1.61-zbd3145"],["v1.60-zbd3145"]]
        nextgen = true
        steamId = "883710"
        runtime = "OpenXR"
        
        [RE3]
        location = "D:/steam/games/RE3"
        version_in_use = "v1.71-abd3145"
        versions = [["v1.71-abd3145"]]
        nextgen = false
        runtime = "OpenXR"
        steamId = "952060"

        [RE7]
        location = "D:/steam/games/RE7"

        [RE8]
        location = "D:/steam/games/RE8"
        version_in_use = "some123"
        versions = [["v1.71-abd3145", "RE8.zip"]]
        runtime = "OpenVR"
        latawiec = true
    "#;
            let (main, games) = deserialize(content).unwrap();
            Ok(REvilConfig { main, games })
        };
        Box::new(default)
    }
}

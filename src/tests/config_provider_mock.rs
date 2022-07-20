#[cfg(test)]
pub mod mock_conf_provider {
    use crate::tomlConf::{
        config::MockConfigProvider, configStruct::REvilConfig, utils::deserialize,
    };

    pub fn get_config_provider_mock() -> Box<MockConfigProvider> {
        let mock_config_provider = MockConfigProvider::new();

        let mut config_provider = Box::new(mock_config_provider);

        // TODO this should be called once but not sure how to make it compatible with dialogs_tests so
        // it is returning instead return_once
        config_provider.expect_load_from_file().returning(|| {
            let content = r#"
        [main]
        sources = ["normal", "nightly"]
        autoupdate = true
        steamExePath = "C:\\Program Files (x86)\\Steam\\steam.exe"

        [RE2]
        location = "D:/steam/games/RE2"
        versions = [["v1.71-abd3145"]]
        nextgen = true
        runtime = "OpenXR"
        
        [RE3]
        location = "D:/steam/games/RE3"
        versions = [["v1.71-abd3145"]]
        nextgen = false
        runtime = "OpenXR"

        [RE7]
        location = "D:/steam/games/RE7"
        versions = [["v1.71-abd3145", "RE7.zip"]]

        [RE8]
        location = "D:/steam/games/RE8"
        versions = [["v1.71-abd3145", "RE8.zip"]]
        runtime = "OpenVR"
        latawiec = true
    "#;
            let (main, games) = deserialize(&content)?;
            Ok(REvilConfig { main, games })
        });
        config_provider.expect_save_to_file().returning(|_| Ok(()));
        config_provider
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use crate::tomlConf::{configStruct::{GameConfig, Main, Runtime, REvilConfig}, utils::{deserialize, serialize}};
    
    #[test]
    fn check_correctness_of_serialization() {
        let content = r#"
        [main]
        sources = ["normal", "nightly"]
        autoupdate = true
        steamExePath = "C:\\Program Files (x86)\\Steam\\steam.exe"

        [RE2]
        location = "D:/steam/games/RE2"
        versions = ["1.71-abd314"]
        nextgen = true
        runtime = "OpenXR"

        [RE7]
        location = "D:/steam/games/RE7"
        versions = ["1.71-abd314"]

        [RE12]
        location = "D:/steam/games/RE12"
        versions = ["1.71-abd314"]
        runtime = "OpenVR"
        latawiec = true
    "#;
        match deserialize(content) {
            Ok((main, games)) => {
                let main_test = Main
                {
                    steamExePath: Some(r"C:\Program Files (x86)\Steam\steam.exe".to_owned()),
                    autoupdate: Some(true),
                    sources: Some(vec!["normal".to_owned(), "nightly".to_owned()]),
                    ..Main::default()
                };
                let mut games_test = HashMap::new();
                // for key in ["RE2", "RE7", "RE12"] {
                let game = GameConfig {
                    steamId: None,
                    location: Some("D:/steam/games/RE2".to_owned()),
                    nextgen: Some(true),
                    runArgs: None,
                    runtime: Some(Runtime::OpenXR),
                    versions: Some(vec!["1.71-abd314".to_owned()]),
                };
                games_test.insert("RE2".to_owned(), game);

                let game = GameConfig {
                    location: Some("D:/steam/games/RE7".to_owned()),
                    nextgen: None,
                    runArgs: None,
                    runtime: None,
                    versions: Some(vec!["1.71-abd314".to_owned()]),
                    steamId: None,
                };
                games_test.insert("RE7".to_owned(), game);

                let game = GameConfig {
                    location: Some("D:/steam/games/RE12".to_owned()),
                    nextgen: None,
                    runArgs: None,
                    runtime: Some(Runtime::OpenVR),
                    versions: Some(vec!["1.71-abd314".to_owned()]),
                    steamId: None,
                };
                games_test.insert("RE12".to_owned(), game);

                assert_eq!(games_test, games);
                assert_eq!(main_test, main);
            }
            Err(err) => panic!("{}", err),
        }
    }

    #[test]
    fn serialize_test() {
        let main_test = Main {
            steamExePath: Some(r"C:\Program Files (x86)\Steam\steam.exe".to_owned()),
            autoupdate: Some(true),
            sources: Some(vec!["normal".to_owned(), "nightly".to_owned()]),
            ..Main::default()
        };
        let mut games_test = HashMap::new();

        let game = GameConfig {
            steamId: None,
            location: Some("D:/steam/games/RE2".to_owned()),
            nextgen: Some(true),
            runArgs: None,
            runtime: Some(Runtime::OpenXR),
            versions: Some(vec!["1.71-abd314".to_owned()]),
        };
        games_test.insert("RE2".to_owned(), game);

        let game = GameConfig {
            steamId: None,
            location: Some("D:/steam/games/RE7".to_owned()),
            nextgen: None,
            runArgs: None,
            runtime: None,
            versions: Some(vec!["1.71-abd314".to_owned()]),
        };
        games_test.insert("RE7".to_owned(), game);

        let game = GameConfig {
            steamId: None,
            location: Some("D:/steam/games/RE12".to_owned()),
            nextgen: None,
            runArgs: None,
            runtime: Some(Runtime::OpenVR),
            versions: Some(vec!["1.71-abd314".to_owned()]),
        };
        games_test.insert("RE12".to_owned(), game);

        let conf = REvilConfig {
            main: main_test,
            games: games_test,
        };
        let content = serialize(&conf).unwrap();

        match deserialize(&content) {
            Ok((main, games)) => {
                assert_eq!(conf.main, main);
                assert_eq!(conf.games, games);
            }
            Err(_) => todo!(),
        };
    }
}

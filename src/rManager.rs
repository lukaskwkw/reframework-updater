use std::{collections::HashMap, error::Error, fmt};

use crate::{
    steam::SteamThings,
    tomlConf::{
        config::ConfigProvider,
        configStruct::{GameConfig, Main, REvilConfig, Runtime, ShortGameName},
    },
    utils::local_version::LocalFiles,
    DynResult,
};
use error_stack::{Context, IntoReport, Report, Result, ResultExt};

#[derive(Debug)]
pub struct REvilManagerError;
impl fmt::Display for REvilManagerError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str("REvilManagerError error")
    }
}

impl Error for REvilManagerError {}

const GAMES: [(&str, &str); 6] = [
    ("601150", "DMC5"),
    ("1446780", "MHRISE"),
    ("883710", "RE2"),
    ("952060", "RE3"),
    ("418370", "RE7"),
    ("1196590", "RE8"),
];

pub struct REvilManager {
    config: REvilConfig,
    config_provider: Box<dyn ConfigProvider>,
    steam_menago: Box<dyn SteamThings>,
    local_provider: Box<dyn LocalFiles>,
}

pub trait REvilThings {
    fn load_config(&mut self) -> Result<&mut Self, REvilManagerError>;
    fn load_games_from_steam(&mut self) -> DynResult<&mut Self>;
    fn get_local_settings_per_game(&mut self) -> &mut Self;
    fn save_config(&mut self) -> DynResult<&mut Self>;
    fn check_for_REFramework_update(&mut self) -> DynResult<&mut Self>;
    fn download_REFramework_update(&mut self) -> DynResult<&mut Self>;
    fn unzip_updates(&mut self) -> DynResult<&mut Self>;
    fn check_for_self_update(&mut self) -> DynResult<&mut Self>;
    fn self_update(&mut self) -> DynResult<&mut Self>;
    fn launch_game(&mut self) -> DynResult<&mut Self>;
}

impl REvilManager {
    pub fn new(
        config_provider: Box<dyn ConfigProvider>,
        local_provider: Box<dyn LocalFiles>,
        steam_menago: Box<dyn SteamThings>,
    ) -> Self {
        Self {
            config: REvilConfig {
                main: Main {
                    sources: None,
                    autoupdate: None,
                    steamExePath: None,
                    steamGamesIdToSearchFor: None,
                },
                games: HashMap::new(),
            },
            config_provider,
            steam_menago,
            local_provider,
        }
    }
}

impl REvilThings for REvilManager {
    fn load_config(&mut self) -> Result<&mut Self, REvilManagerError> {
        let config = self
            .config_provider
            .load_from_file()
            .change_context(REvilManagerError)?;
        self.config = config;
        Ok(self)
    }

    fn get_local_settings_per_game(&mut self) -> &mut Self {
        for (_, config) in self.config.games.iter_mut() {
            let game_location = config.location.as_ref().unwrap();
            let local_config = match self
                .local_provider
                .get_local_report_for_game(&game_location)
            {
                Ok(it) => it,
                Err(err) => {
                    println!("{}", err);
                    continue;
                }
            }; //.attach_printable(|| format!("something")).p;
               // .unwrap();

            config.runtime = local_config.runtime;
            config.versions = Some([local_config.version.unwrap()].to_vec());
        }
        println!("Full config: \n {:#?}", self.config);
        self
    }

    fn load_games_from_steam(&mut self) -> DynResult<&mut Self> {
        println!("Going to auto-detect games");
        let game_ids = GAMES.map(|(k, v)| k);
        let games_tuple_arr = self.steam_menago.get_games_locations(&game_ids.to_vec())?;

        games_tuple_arr.iter().for_each(|(id, path)| {
            let (_, game_short_name) = GAMES.iter().find(|(game_id, _)| game_id == id).unwrap();

            #[cfg(debug_assertions)]
            println!("game detected \n name {}, \n path {:?}", game_short_name, path);

            self.config.games.insert(
                game_short_name.to_string(),
                GameConfig {
                    location: Some(path.display().to_string()),
                    steamId: Some(id.to_owned()),
                    versions: None,
                    nextgen: Some(false),
                    runtime: Some(Runtime::OpenVR),
                    runArgs: None,
                },
            );
        });

        Ok(self)
    }

    fn save_config(&mut self) -> DynResult<&mut Self> {
        todo!()
    }

    fn check_for_REFramework_update(&mut self) -> DynResult<&mut Self> {
        todo!()
    }

    fn download_REFramework_update(&mut self) -> DynResult<&mut Self> {
        todo!()
    }

    fn unzip_updates(&mut self) -> DynResult<&mut Self> {
        todo!()
    }

    fn check_for_self_update(&mut self) -> DynResult<&mut Self> {
        todo!()
    }

    fn self_update(&mut self) -> DynResult<&mut Self> {
        todo!()
    }

    fn launch_game(&mut self) -> DynResult<&mut Self> {
        todo!()
    }
}

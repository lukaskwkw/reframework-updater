#[cfg(target_os = "windows")]
use rManager::{REvilManager};
use strategy::StrategyFactory::StrategyFactory;
use std::{
    error::{self, Error},
};
use steam::SteamManager;
use tomlConf::config::{REvilConfigProvider};
use utils::local_version::LocalProvider;

mod refr_github;
mod utils {
    pub mod binSearch;
    pub mod fetch;
    pub mod local_version;
    pub mod version_parser;
    pub mod mslink;
}

mod steam;
pub mod unzip {
    pub mod UnzipError;
    pub mod unzip;
}
mod tests {
    pub mod data;
}
pub mod strategy {
    pub mod StrategyFactory;
}
mod rManager;
mod tomlConf {
    pub mod FromValue;
    pub mod config;
    pub mod configStruct;
    pub mod configTest;
    pub mod utils;
}

pub type DynResult<T> = Result<T, Box<dyn Error>>;

const NIGHTLY_RELEASES: &str = "https://github.com/praydog/REFramework-nightly/releases";
static GAMES: [(&str, &str); 6] = [
    ("601150", "DMC5"),
    ("1446780", "MHRISE"),
    ("883710", "RE2"),
    ("952060", "RE3"),
    ("418370", "RE7"),
    ("1196590", "RE8"),
];

// #[tokio::main]
fn main() -> Result<(), Box<dyn error::Error>> {
    let config_provider = Box::new(REvilConfigProvider::new("config.toml"));
    let steam_menago = Box::new(SteamManager);
    let local_provider = Box::new(LocalProvider);
    let mut evilManager = REvilManager::new(config_provider, local_provider, steam_menago);
    
    let strategy = StrategyFactory::get_strategy(&mut evilManager);
    strategy(&mut evilManager);
    Ok(())
}

/* TODO game.exe STEAMAppID
MonsterHunterRise.exe 1446780
DevilMayCry5.exe 601150
re7.exe 418370
re8.exe 1196590
re3.exe 952060
re2.exe 883710
*/

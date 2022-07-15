use args::ArgsClap;

use dialogs::dialogs::Dialogs;
#[cfg(target_os = "windows")]
use rManager::rManager_header::REvilManager;
use reframework_github::refr_github::{self, REFRGithub};

use core::time;
use std::{error::{self, Error}, thread};
use steam::SteamManager;
use strategy::StrategyFactory::StrategyFactory;
use tomlConf::config::REvilConfigProvider;
use utils::local_version::LocalProvider;

pub mod reframework_github {
    pub mod refr_github;
    pub mod release;
}
mod utils {
    pub mod binSearch;
    pub mod fetch;
    pub mod init_logger;
    pub mod local_version;
    pub mod mslink;
    pub mod progress_style;
    pub mod version_parser;
    pub mod find_game_conf_by_steam_id;
    pub mod is_asset_tdb;
}

mod steam;
pub mod dialogs {
    pub mod dialogs;
    pub mod dialogs_label;
}
pub mod unzip {
    pub mod UnzipError;
    pub mod unzip;
}
// mod tests {
//     pub mod data;
// }
pub mod strategy {
    pub mod StrategyFactory;
}
mod args;
mod rManager {
    pub mod rManager;
    pub mod rManager_header;
    pub mod cleanup_cache;
}
mod tomlConf {
    pub mod FromValue;
    pub mod config;
    pub mod configStruct;
    pub mod configTest;
    pub mod utils;
}

pub type DynResult<T> = Result<T, Box<dyn Error>>;

static NIGHTLY_RELEASE: &str = "REFramework-nightly";
static REPO_OWNER: &str = "praydog";
static GAMES: [(&str, &str); 6] = [
    ("601150", "DMC5"),
    ("1446780", "MHRISE"),
    ("883710", "RE2"),
    ("952060", "RE3"),
    ("418370", "RE7"),
    ("1196590", "RE8"),
];
static GAMES_NEXTGEN_SUPPORT: [&str; 3] = ["RE2", "RE3", "RE7"];

static mut ARGS: Option<ArgsClap> = None;

static STANDARD_TYPE_QUALIFIER: &str = "_TDB";

static MAX_ZIP_FILES_PER_GAME_CACHE: u8 = 3;

pub fn create_TDB_string(game_short_name: &str) -> String {
    format!("{}{}", game_short_name, STANDARD_TYPE_QUALIFIER)
}

// #[tokio::main]
fn main() -> Result<(), Box<dyn error::Error>> {
    let config_provider = Box::new(REvilConfigProvider::new("config.toml"));
    let steam_menago = Box::new(SteamManager);
    let local_provider = Box::new(LocalProvider);
    let dialogs = Box::new(Dialogs);
    let mut evilManager = REvilManager::new(
        config_provider,
        local_provider,
        steam_menago,
        dialogs,
        REFRGithub::new,
    );

    let strategy = StrategyFactory::get_strategy(&mut evilManager);
    strategy(&mut evilManager);
    // let secs = time::Duration::from_secs(20);
    // thread::sleep(secs);
    Ok(())
}

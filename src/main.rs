use args::{ArgsClap, parse_args};
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
    pub mod get_local_path_to_cache;
    pub mod binSearch;
    pub mod fetch;
    pub mod init_logger;
    pub mod local_version;
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
pub mod unzip;

#[cfg(test)]
mod tests {
    pub mod integration;
    pub mod config_provider_mock;
    pub mod steam_mock;
    pub mod refr_github_mock;
    pub mod dialog_provider_mock;
}

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

// #[tokio::main]
fn main() -> Result<(), Box<dyn error::Error>> {
    unsafe {
        parse_args();
    };
    let config_provider = Box::new(REvilConfigProvider::new("config.toml"));
    let steam_menago = Box::new(SteamManager);
    let local_provider = Box::new(LocalProvider);
    let dialogs = Box::new(Dialogs);
    let mut evil_manager = REvilManager::new(
        config_provider,
        local_provider,
        steam_menago,
        dialogs,
        REFRGithub::new,
    );

    let strategy = StrategyFactory::get_strategy(&mut evil_manager);
    strategy(&mut evil_manager);
    let secs = time::Duration::from_secs(7);
    thread::sleep(secs);
    Ok(())
}
// TODO implement back functionality
// TODO implement not run game after any switching option
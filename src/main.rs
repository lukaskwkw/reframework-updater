use args::{parse_args, ArgsClap};
use dialogs::dialogs::Dialogs;
use log::info;
#[cfg(target_os = "windows")]
use rManager::rManager_header::REvilManager;
use reframework_github::refr_github::{self, REFRGithub};

use core::time;
use std::{
    error::{self, Error},
    thread,
};
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
    pub mod find_game_conf_by_steam_id;
    pub mod get_local_path_to_cache;
    pub mod init_logger;
    pub mod is_asset_tdb;
    pub mod local_version;
    pub mod open_dialog;
    pub mod progress_style;
    pub mod restart_program;
    pub mod version_parser;
}

mod steam;
pub mod dialogs {
    pub mod dialogs;
    pub mod dialogs_label;
}
pub mod unzip;

#[cfg(test)]
mod tests {
    pub mod config_provider_mock;
    pub mod dialog_provider_mock;
    pub mod init_dialogs_mock;
    pub mod integration;
    pub mod local_provider_mock;
    pub mod manager_mocks;
    pub mod refr_github_mock;
    pub mod steam_mock;
}

pub mod strategy {
    pub mod StrategyFactory;
}
mod args;
mod rManager {
    pub mod cleanup_cache;
    pub mod rManager;
    pub mod rManager_header;
}
mod tomlConf {
    pub mod FromValue;
    pub mod config;
    pub mod configStruct;
    #[cfg(test)]
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

static MAX_ZIP_FILES_PER_GAME_CACHE: u8 = 4;

static TIME_TO_CLOSE: u16 = 10;
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
    info!("This window will close after {} seconds", TIME_TO_CLOSE);
    let secs = time::Duration::from_secs(TIME_TO_CLOSE as u64);
    thread::sleep(secs);
    Ok(())
}
// TODO Broken download/unzip doesn't no longer alter the game mod config. But maybe we can
//      also add option to not launch not updated/unzipped correctly mod. I.e. can be implemented as follow
//      when selected_game_to_launch is same as for broken download/unzip then terminate process after prompt inside
//      or press anything on keyboard to continue

// TODO add saving logs by using simple-log
// TODO maybe instead terminating catchable on error show that error and then ask for press key to exit?
// TODO when using rescan option, local config can contain different version so it is added to beginning of array versions but then when switching different version from cache
//      that particular mod from local config is gone now -> to fix that we should implement zipping local version after any detection or prevent to loading cache for that game (1st preferable)
//      this bug should not occurs often..
// TODO be aware that when implementing zipping functionality runtime file can be missing and it can complicate later runtime switching!

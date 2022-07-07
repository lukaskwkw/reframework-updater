use args::ArgsClap;
use clap::Parser;
#[cfg(target_os = "windows")]
use rManager::REvilManager;
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
    pub mod local_version;
    pub mod mslink;
    pub mod progress_style;
    pub mod version_parser;
    pub mod init_logger;
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
mod args;
mod rManager;
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

static mut ARGS: Option<ArgsClap> = None;

// #[tokio::main]
fn main() -> Result<(), Box<dyn error::Error>> {
    let config_provider = Box::new(REvilConfigProvider::new("config.toml"));
    let steam_menago = Box::new(SteamManager);
    let local_provider = Box::new(LocalProvider);
    let mut evilManager = REvilManager::new(
        config_provider,
        local_provider,
        steam_menago,
        REFRGithub::new,
    );

    let strategy = StrategyFactory::get_strategy(&mut evilManager);
    strategy(&mut evilManager);
    Ok(())
}

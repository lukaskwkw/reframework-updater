#![feature(option_result_contains)]

use crate::refr_github::REFRGithub;
use crate::utils::{fetch::fetch_file, local_version, version_parser};
// use env_logger::Env;
// use log::{debug, info, trace, warn};
#[cfg(target_os = "windows")]
use mslnk::ShellLink;
use rManager::{REvilManager, REvilThings};
use strategy::StrategyFactory::StrategyFactory;
use std::path::{Path, PathBuf};
use std::{
    error::{self, Error},
    thread,
    time::Duration,
};
use std::{fs, vec};
use steam::SteamManager;
use tomlConf::config::{self, ConfigProvider, REvilConfigProvider};
use unzip::unzip::unzip;
use utils::local_version::LocalProvider;

mod refr_github;
mod utils {
    pub mod binSearch;
    pub mod fetch;
    pub mod local_version;
    pub mod version_parser;
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

#[cfg(target_os = "windows")]
pub fn create_ms_lnk(target: impl AsRef<Path>, arguments: Option<String>) {
    // let target = r"C:\Program Files\Mozilla Firefox\firefox.exe";
    let lnk = "npad.lnk";
    let mut sl = ShellLink::new(target).unwrap();
    sl.set_arguments(arguments);
    // sl.set_arguments(Some("-private-window".to_owned()));
    sl.create_lnk(lnk).unwrap();
}

/*     game.commands.launch = Some(vec![
        launcher_executable.display().to_string(),
        String::from("-silent"),
        format!("steam://run/{}", &game.id),
    ]);
*/

// #[tokio::main]
fn main() -> Result<(), Box<dyn error::Error>> {
    let config_provider = Box::new(REvilConfigProvider::new("config.toml"));
    let steam_menago = Box::new(SteamManager);
    let local_provider = Box::new(LocalProvider);
    let mut evilManager = REvilManager::new(config_provider, local_provider, steam_menago);
    
    let strategy = StrategyFactory::get_strategy(&mut evilManager);
    strategy(&mut evilManager);
    // evilManager.check_for_self_update();
    Ok(())
}

/* TODO game.exe STEAMAppID
MonsterHunterRise.exe 1446780
DevilMayCry5.exe 601150
re7.exe 418370
re8.exe 1196590
re3.exe 952060
re2.exe 883710

DMC5
MHRISE
RE2
RE3
RE7
RE8
*/

#![feature(option_result_contains)]

use crate::refr_github::REFRGithub;
use crate::steam::get_games_locations;
use crate::toml::config::{deserialize, serialize};
use crate::utils::{fetch::fetch_file, local_version::getLocalVersions, version_parser};
#[cfg(target_os = "windows")]
use mslnk::ShellLink;
use scrapper::scrapper::scrape_latest_data;
use std::fs;
use std::path::{Path, PathBuf};
use std::{
    error::{self, Error},
    thread,
    time::Duration,
};
use unzip::unzip::unzip;

mod refr_github;
mod utils {
    pub mod fetch;
    pub mod local_version;
    pub mod version_parser;
}
mod scrapper {
    pub mod ScrapperError;
    pub mod scrapper;
    pub mod scrapperTest;
}
mod steam;
pub mod unzip {
    pub mod UnzipError;
    pub mod unzip;
}
mod tests {
    pub mod data;
}
mod toml {
    pub mod FromValue;
    pub mod config;
}

pub type VerResult = Result<String, Box<dyn Error>>;

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

// #[tokio::main]
fn main() -> Result<(), Box<dyn error::Error>> {
    // let file_content = std::fs::read_to_string("./src/tests/releases_nightly.htm").unwrap();
    let args: Vec<_> = std::env::args().collect();
    for argument in std::env::args() {
        println!("Argument {}", argument);
    }
    if args.len() > 2 && args[1] == "-run" {
        print!("About to run {} are you happy now?!:) \n", args[2]);
    }

    let files_to_skip = [
        "openvr_api.dll",
        "DELETE_OPENVR_API_DLL_IF_YOU_WANT_TO_USE_OPENXR",
    ];

    println!("Before");
    let ref_github = REFRGithub::new("REFramework-nightly", "praydog", "TDB");

    let release = ref_github.get_reframework_latest_release()?;
    println!("release TBD{:?}", release);
    // download_release_asset(&release.assets.first().unwrap());
    // println!("Before RTX");
    // let release = get_reframework_latest_release("REFramework-nightly")?;
    // println!("{:?}", release.assets);

    // if isRepoVersionNewer(release.name, );
    // let config = ReleaseListBuilder::
    // let mut listBuilder = github::ReleaseList::configure();
    // let list = listBuilder
    //     .repo_name("REFramework-nightly")
    //     .repo_owner("praydog")
    //     .build()
    //     .unwrap();

    // let releases = list.fetch().unwrap();
    // println!("{:?}", first);

    // let list = t.build().unwrap();
    // t.repo_name("REFramework-nightly").repo_owner("praydog");
    // let executable = PathBuf::from(args.get(0).unwrap());
    // let executable = PathBuf::from(args.get(0).unwrap());

    #[cfg(target_os = "windows")]
    create_ms_lnk(&args[0], Some("-run RE7".to_owned()));
    // let steam_folder = "C:\\Program Files (x86)\\Steam";
    // getLibraryFoldersFile(steam_folder);
    let ids = vec!["1446780", "601150", "418370", "1196590", "952060", "883710"];
    let paths = get_games_locations(ids).unwrap();
    let re = paths.first();
    println!("{:?}", re);

    getLocalVersions(&paths)
        .unwrap()
        .iter()
        .for_each(|p| println!("{:?}", p.as_ref().unwrap()));

    // unzip(files_to_skip.to_vec(), re, false).unwrap();
    // let (scraped_links, _timestamps) = match scrape_latest_data(file_content) {
    //     Ok((scraped_links, timestamps)) => (scraped_lin`ks, timestamps),
    //     Err(err) => {
    //         // runGame() // runGame anyway
    //         return Err(err.to_string())?;
    //     }
    // };

    // fetch_file(&scraped_links[0]).await?;

    // serialize();
    deserialize().unwrap();

    println!("what");
    thread::sleep(Duration::from_secs(30));

    // fetch_file(&scraped_links[6], None).await?;

    // scraped_links.iter().for_each(|link| println!("{}", link));
    // timestamps
    //     .iter()
    //     .for_each(|date_time| println!("{}", date_time));

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
RE2_TDB66
RE3
RE3_TDB67
RE7
RE7_TDB49
RE8
*/

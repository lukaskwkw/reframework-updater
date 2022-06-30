#![feature(option_result_contains)]

use crate::{steam::get_games_locations};
use crate::utils::{local_version::getLocalVersions, version_parser, fetch::fetch_file};
use crate::toml::config::{deserialize, serialize};
use scrapper::scrapper::scrape_latest_data;
use std::{
    error::{self, Error},
    thread,
    time::Duration,
};
use unzip::unzip::unzip;

mod utils {
    pub mod local_version;
    pub mod version_parser;
    pub mod fetch;
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
    pub mod config;
    pub mod FromValue;
}

pub type VerResult = Result<String, Box<dyn Error>>;

pub type DynResult<T> = Result<T, Box<dyn Error>>;

const NIGHTLY_RELEASES: &str = "https://github.com/praydog/REFramework-nightly/releases";

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    // let file_content = std::fs::read_to_string("./src/tests/releases_nightly.htm").unwrap();

    let files_to_skip = [
        "openvr_api.dll",
        "DELETE_OPENVR_API_DLL_IF_YOU_WANT_TO_USE_OPENXR",
    ];

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
*/

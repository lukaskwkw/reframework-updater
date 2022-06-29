#![feature(option_result_contains)]

use config::{deserialize, serialize};
use fetch::fetch_file;

use scrapper::scrapper::scrape_latest_data;
use std::error;

mod FromValue;
mod scrapper {
    pub mod ScrapperError;
    pub mod scrapper;
}

pub mod unzip {
    pub mod UnzipError;
    pub mod unzip;
}
mod tests {
    pub mod data;
}

mod fetch;

mod config;

const NIGHTLY_RELEASES: &str = "https://github.com/praydog/REFramework-nightly/releases";

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    // let file_content = std::fs::read_to_string("./src/tests/releases_nightly.htm").unwrap();

    let _files_to_skip = [
        "openvr_api.dll",
        "DELETE_OPENVR_API_DLL_IF_YOU_WANT_TO_USE_OPENXR",
    ];
    // unzip(files_to_skip.to_vec(), true).unwrap();
    // let (scraped_links, _timestamps) = match scrape_latest_data(file_content) {
    //     Ok((scraped_links, timestamps)) => (scraped_links, timestamps),
    //     Err(err) => {
    //         // runGame() // runGame anyway
    //         return Err(err.to_string())?;
    //     }
    // };

    // fetch_file(&scraped_links[0]).await?;

    // serialize();
    deserialize().unwrap();

    println!("what");

    // fetch_file(&scraped_links[6], None).await?;

    // scraped_links.iter().for_each(|link| println!("{}", link));
    // timestamps
    //     .iter()
    //     .for_each(|date_time| println!("{}", date_time));

    Ok(())
}

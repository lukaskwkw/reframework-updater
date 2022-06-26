#![feature(option_result_contains)]

use error_chain::error_chain;
use scrapper::scrape_latest_data;

mod scrapper;
mod tests {
    pub mod data;
}

error_chain! {
      foreign_links {
          ReqError(reqwest::Error);
          IoError(std::io::Error);
      }
}

const NIGHTLY_RELEASES: &str = "https://github.com/praydog/REFramework-nightly/releases";

#[tokio::main]
async fn main() -> Result<()> {
    let file_content = std::fs::read_to_string("./src/releases_nightly.htm").unwrap();

    let (links, date_time_list) = scrape_latest_data(file_content);

    links.iter().for_each(|link| println!("{}", link));
    date_time_list
        .iter()
        .for_each(|date_time| println!("{}", date_time));

    Ok(())
}

use std::{collections::HashMap, fs};

use reqwest::header;
use self_update::{
    backends::github::{self, ReleaseListBuilder},
    update::{Release, ReleaseAsset},
    Download,
};

use crate::DynResult;

#[derive(Clone, Debug, Default)]
pub struct REFRGithub {
    repo_name: String,
    repo_owner: String,
    filter_target: String,
}

impl REFRGithub {
    pub fn new(repo_name: &str, repo_owner: &str, filter_target: &str) -> Self {
        REFRGithub {
            repo_name: repo_name.to_owned(),
            repo_owner: repo_owner.to_owned(),
            filter_target: filter_target.to_owned(),
        }
    }

    pub fn get_reframework_latest_release(&self) -> DynResult<Release> {
        let releases = self_update::backends::github::ReleaseList::configure()
            .repo_name(&self.repo_name)
            .repo_owner(&self.repo_owner)
            .build()?
            .fetch()?;

        let first = releases.first().ok_or("Releases not found")?;
        // let first = releases.first().ok_or("Releases not found")?;
        // self.release = Some(first);
        return Ok(first.clone());
    }

    pub fn download_release_asset(self, release_asset: &ReleaseAsset) -> DynResult<Self> {
        let mut download = Download::from_url(&release_asset.download_url);
        let mut headers = header::HeaderMap::new();
        headers.insert(header::ACCEPT, "application/octet-stream".parse().unwrap());
        download.set_headers(headers);

        download.show_progress(true);

        let mut tmp_archive = fs::File::create(&release_asset.name)?;
        download.download_to(&mut tmp_archive)?;
        Ok(self)
    }

    // pub fn set_filtered_assets_as_values(
    //     &self,
    //     games
    //     // games_hash: &mut HashMap<String, String>,
    //     should_contain: &bool,
    // ) -> DynResult<()> {
    //     let
    //     for game in games_hash  {
    //         game.
    //     }

    //         .for
    //         .filter(|asset| asset.download_url.contains(self.filter_target) == should_contain);
    //         ()
    // }
}

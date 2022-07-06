use core::fmt;
use std::{
    collections::HashMap,
    error::Error,
    fmt::{Display, Formatter},
    fs, path::Path,
};

use error_stack::{bail, IntoReport, Report, ResultExt};
use log::{debug, error};
use reqwest::header;
use self_update::{
    backends::github::{self, ReleaseListBuilder},
    update::{Release, ReleaseAsset},
    Download,
};

use crate::{DynResult, utils::fetch::fetch_release};

use super::release::ReleaseREFR;

#[derive(Clone, Debug, Default)]
pub struct REFRGithub {
    repo_name: String,
    repo_owner: String,
    filter_target: String,
    pub release: Option<Release>,
}

#[derive(Debug)]
pub struct REFRGithubError;

impl Display for REFRGithubError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        fmt.write_str("REFRGithub Error")
    }
}

impl Error for REFRGithubError {}

// type DynResult<T> = Result<T, REFRGithubError>;

pub trait ManageGithub<T = REFRGithub> {
    fn get_reframework_latest_release(&mut self) -> DynResult<()>;
    fn download_release_asset(&self, release_asset: &ReleaseAsset) -> DynResult<&T>;
    fn fetch_release(&self) -> DynResult<Release>;
    // fn filter_ou(&self) -> DynResult<Release>;
    fn getRelease(&self) -> Option<&Release>;
}

impl ManageGithub for REFRGithub {
    fn get_reframework_latest_release(&mut self) -> DynResult<()> {
        let release = self.fetch_release()?;
        debug!("Release {:?}", release);
        self.release = Some(release);
        return Ok(());
    }

    fn download_release_asset(&self, release_asset: &ReleaseAsset) -> DynResult<&Self> {
        // self.release.assets
        let mut download = Download::from_url(&release_asset.download_url);
        let mut headers = header::HeaderMap::new();
        headers.insert(header::ACCEPT, "application/octet-stream".parse().unwrap());
        download.set_headers(headers);

        download.show_progress(true);

        // TODO path should be like ./releases/v1.202-0121f55/<name> let path = Path::new(release_asset.)
        let mut tmp_archive = fs::File::create(&release_asset.name)?;
        download.download_to(&mut tmp_archive)?;
        Ok(self)
    }

    fn fetch_release(&self) -> DynResult<Release> {
        let api_url = format!(
            "{}/repos/{}/{}/releases",
            "https://api.github.com", self.repo_owner, self.repo_name
        );
        let release = fetch_release(&api_url)?;
        return Ok(release);
    }

    fn getRelease(&self) -> Option<&Release> {
        self.release.as_ref()
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

impl REFRGithub {
    pub fn new(repo_owner: &str, repo_name: &str, filter_target: &str) -> Self {
        REFRGithub {
            repo_owner: repo_owner.to_owned(),
            repo_name: repo_name.to_owned(),
            filter_target: filter_target.to_owned(),
            release: None,
        }
    }
}

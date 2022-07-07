use core::fmt;
use std::{
    collections::HashMap,
    error::Error,
    fmt::{format, Display, Formatter},
    fs,
    ops::ControlFlow,
    path::Path,
};

use error_stack::{bail, IntoReport, Report, ResultExt};
use log::{debug, error, trace};
use reqwest::header;
use self_update::{
    backends::github::{self, ReleaseListBuilder},
    update::{Release, ReleaseAsset},
    Download,
};

use crate::{utils::fetch::fetch_release_api, DynResult, GAMES_NEXTGEN_SUPPORT};

use super::release::ReleaseREFR;

type GameShortName = String;

#[derive(Clone, Debug, Default)]
pub struct REFRGithub {
    repo_name: String,
    repo_owner: String,
    filter_target: String,
    pub release: Option<Release>,
    pub report: HashMap<GameShortName, Vec<ReleaseAsset>>,
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
    fn generate_assets_report(&mut self) -> DynResult<()>;
    fn download_release_asset(&self, release_asset: &ReleaseAsset) -> DynResult<&T>;
    fn fetch_release(&self) -> DynResult<Release>;
    // fn filter_ou(&self) -> DynResult<Release>;
    fn getRelease(&self) -> Option<&Release>;
    fn getAssetsReport(&self) -> &HashMap<GameShortName, Vec<ReleaseAsset>>;
}

impl ManageGithub for REFRGithub {
    fn get_reframework_latest_release(&mut self) -> DynResult<()> {
        let release = self.fetch_release()?;
        trace!("{:?}", release);
        self.release = Some(release);
        self.generate_assets_report()?;
        trace!("Assets Report: {:#?}", self.report);
        return Ok(());
    }

    fn generate_assets_report(&mut self) -> DynResult<()> {
        let assets = &self.release.as_ref().ok_or("Release not found")?.assets;
        assets.iter().try_for_each(|asset| -> DynResult<()> {
            let game_short_name = GAMES_NEXTGEN_SUPPORT
                .iter()
                .find(|short_name| asset.name.contains(*short_name));
            if let Some(it) = game_short_name {
                self.report
                    .entry(it.to_string())
                    .and_modify(|assets| assets.push(asset.clone()))
                    .or_insert([asset.clone()].to_vec());
                ()
            } else {
                let short_name = asset.name.split('.').collect::<Vec<&str>>();
                let short_name = short_name.first().ok_or(format!(
                    "asset name doesn't follow <%s>.<%s> format i.e. should be RE7.zip found [{}]",
                    asset.name
                ))?;

                self.report
                    .insert(short_name.to_string(), [asset.clone()].to_vec());
            }
            Ok(())
        })?;
        Ok(())
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
        let release = fetch_release_api(&api_url)?;
        return Ok(release);
    }

    fn getRelease(&self) -> Option<&Release> {
        self.release.as_ref()
    }

    fn getAssetsReport(&self) -> &HashMap<std::string::String, Vec<ReleaseAsset>> {
        &self.report
    }
}

impl REFRGithub {
    pub fn new(repo_owner: &str, repo_name: &str, filter_target: &str) -> Self {
        REFRGithub {
            repo_owner: repo_owner.to_owned(),
            repo_name: repo_name.to_owned(),
            filter_target: filter_target.to_owned(),
            release: None,
            report: HashMap::new(),
        }
    }
}

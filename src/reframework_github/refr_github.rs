#[cfg(test)]
use mockall::automock;

use core::fmt;
use std::{
    collections::HashMap,
    error::Error,
    fmt::{Display, Formatter},
    fs,
    path::PathBuf,
};

use error_stack::IntoReport;
use log::{info, trace};
use reqwest::header;
use self_update::{
    update::{Release, ReleaseAsset},
    Download,
};

use crate::{
    utils::{fetch::fetch_release_api, get_local_path_to_cache::get_local_path_to_cache_folder},
    DynResult, GAMES_NEXTGEN_SUPPORT,
};

pub type GameShortName = String;

pub type AssetsReport = HashMap<GameShortName, Vec<ReleaseAsset>>;

#[derive(Clone, Debug, Default)]
pub struct REFRGithub {
    repo_name: String,
    repo_owner: String,
    pub release: Option<Release>,
    pub report: AssetsReport,
}

#[derive(Debug)]
pub enum REFRGithubError {
    VersionIsNoneAndReleaseIsNone,
}

impl Display for REFRGithubError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        fmt.write_str("REFRGithub Error")
    }
}

impl Error for REFRGithubError {}

pub trait ManageGithub<T = REFRGithub> {
    fn get_reframework_latest_release(&mut self) -> DynResult<()>;
    fn generate_assets_report(&mut self) -> DynResult<()>;
    fn download_release_asset(&self, release_asset: &ReleaseAsset) -> DynResult<&T>;
    fn fetch_release(&self) -> DynResult<Release>;
    fn getRelease(&self) -> Option<&Release>;
    fn getAssetsReport(&self) -> &AssetsReport;
}

impl ManageGithub for REFRGithub {
    fn get_reframework_latest_release(&mut self) -> DynResult<()> {
        let release = self.fetch_release()?;
        trace!("{:?}", release);
        self.release = Some(release);
        self.generate_assets_report()?;
        trace!("Assets Report: {:#?}", self.report);
        Ok(())
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

    // TODO return value should be changed to just DynResult<()> as there is no need to return Self. It makes testing complicated
    fn download_release_asset(&self, release_asset: &ReleaseAsset) -> DynResult<&Self> {
        let mut download = Download::from_url(&release_asset.download_url);
        let mut headers = header::HeaderMap::new();
        headers.insert(header::ACCEPT, "application/octet-stream".parse().unwrap());
        download.set_headers(headers);

        download.show_progress(true);
        let folders = get_local_path_to_cache_folder(self.release.as_ref(), None)?;
        fs::create_dir_all(&folders).map_err(|err| {
            format!(
                "Error during create_dir_all path {} Err {}",
                folders.display(),
                err
            )
        })?;

        let path = &folders.join(&release_asset.name);
        info!("Downloading {} to {}", release_asset.name, path.display());
        let mut tmp_archive = fs::File::create(&path).map_err(|err| {
            format!(
                "Error during File::create. path {} Err {}",
                path.display(),
                err
            )
        })?;

        download.download_to(&mut tmp_archive)?;
        Ok(self)
    }

    fn fetch_release(&self) -> DynResult<Release> {
        let api_url = format!(
            "{}/repos/{}/{}/releases",
            "https://api.github.com", self.repo_owner, self.repo_name
        );
        let release = fetch_release_api(&api_url)?;
        Ok(release)
    }

    fn getRelease(&self) -> Option<&Release> {
        self.release.as_ref()
    }

    fn getAssetsReport(&self) -> &HashMap<std::string::String, Vec<ReleaseAsset>> {
        &self.report
    }
}

#[cfg_attr(test, automock)]
impl REFRGithub {
    pub fn new(repo_owner: &str, repo_name: &str) -> Box<dyn ManageGithub> {
        Box::new(REFRGithub {
            repo_owner: repo_owner.to_owned(),
            repo_name: repo_name.to_owned(),
            release: None,
            report: HashMap::new(),
        })
    }
}

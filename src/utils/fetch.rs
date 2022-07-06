use std::io::{Cursor};
use std::{
    error::Error,
};

use error_stack::{bail, IntoReport, Report, ResultExt};
use log::{debug, error};
use reqwest::header;
use self_update::{
    update::{Release}
};

use crate::DynResult;
use crate::reframework_github::release::ReleaseREFR;

use reqwest::Url;

pub async fn fetch_file(url: &str, file_name: Option<String>) -> Result<(), Box<dyn Error>> {
    let response = reqwest::get(url).await?;

    let f: String = file_name.unwrap_or(getFileNameFromUrl(url)?);
    // response.headers().iter().for_each(|h| println!("{:?}", h));
    let mut file = std::fs::File::create(f)?;
    let mut content = Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    Ok(())
}

fn getFileNameFromUrl(url: &str) -> Result<String, Box<dyn Error>> {
    let url = Url::parse(url)?;
    let fname = url
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("tmp.zip");

    Ok(fname.to_owned())
}

pub fn fetch_release(github_api_url: &str) -> DynResult<Release> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        "rust-reqwest/reframework-update"
            .parse()
            .expect("github invalid user-agent"),
    );

    let resp = reqwest::blocking::Client::new()
        .get(github_api_url)
        .headers(headers)
        .send()?;
    if !resp.status().is_success() {
        error!(
            "api request failed with status: {:?} - for: {:?}",
            resp.status(),
            github_api_url
        )
    }

    let releases = resp.json::<serde_json::Value>()?;
    let releases = releases
        .as_array()
        .ok_or_else(|| format!("No releases found"))?;
    let releases = releases
        .iter()
        .take(1)
        .map(ReleaseREFR::from_release)
        .collect::<DynResult<Vec<Release>>>()?;
    let release = releases
        .first()
        .ok_or_else(|| format!("No release found"))?;
    return Ok(release.clone());
}

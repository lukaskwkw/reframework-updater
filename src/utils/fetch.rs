use std::io::{Cursor};
use std::{
    error::Error,
};
use log::{error};
use reqwest::header;
use self_update::{
    update::{Release}
};
use crate::DynResult;
use crate::reframework_github::release::ReleaseREFR;

pub fn fetch_release_api(github_api_url: &str) -> DynResult<Release> {
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
        .ok_or_else(|| "No releases found".to_string())?;
    let releases = releases
        .iter()
        .take(1)
        .map(ReleaseREFR::from_release)
        .collect::<DynResult<Vec<Release>>>()?;
    let release = releases
        .first()
        .ok_or_else(|| "No release found".to_string())?;
    Ok(release.clone())
}

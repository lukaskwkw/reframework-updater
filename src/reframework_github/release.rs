use self_update::update::{Release, ReleaseAsset};

use crate::DynResult;

pub struct ReleaseREFR;
impl ReleaseREFR {
    pub fn from_release(release: &serde_json::Value) -> DynResult<Release> {
        let tag = release["tag_name"]
            .as_str()
            .ok_or("Release missing `tag_name`")?;
        let date = release["created_at"]
            .as_str()
            .ok_or("Release missing `created_at`")?;
        let name = release["name"].as_str().unwrap_or(tag);
        let assets = release["assets"].as_array().ok_or("No assets found")?;
        let body = release["body"].as_str().map(String::from);
        let assets = assets
            .iter()
            .map(ReleaseREFR::from_asset)
            .collect::<DynResult<Vec<ReleaseAsset>>>()?;
        Ok(Release {
            name: name.to_owned(),
            version: tag.trim_start_matches('v').to_owned(),
            date: date.to_owned(),
            body,
            assets,
        })
    }
    fn from_asset(asset: &serde_json::Value) -> DynResult<ReleaseAsset> {
        let download_url = asset["url"].as_str().ok_or("Asset missing `url`")?;
        let name = asset["name"].as_str().ok_or("Asset missing `name`")?;
        Ok(ReleaseAsset {
            download_url: download_url.to_owned(),
            name: name.to_owned(),
        })
    }
}

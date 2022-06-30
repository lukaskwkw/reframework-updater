use select::document::Document;
use select::node::Node;
use select::predicate::{Class, Name};

use crate::scrapper::ScrapperError::ScrapperError;
use crate::version_parser::isRepoVersionNewer;

const GITHUB_URL: &str = "https://github.com";

pub fn scrape_latest_data(
    html: String,
    local_version: &str,
) -> Result<(Vec<String>, String), ScrapperError> {
    let document = Document::from(html.as_str());

    let repo_version_collection = document
        .find(Class("Link--primary"))
        .take(1)
        .map(|node| node.text())
        .collect::<Vec<String>>();

    let repo_version = repo_version_collection
        .first()
        .ok_or(ScrapperError::GetRepoVersion)?;

    if !isRepoVersionNewer(&local_version, repo_version).ok_or(ScrapperError::VersionParserError)? {
        return Err(ScrapperError::NothingToUpdate);
    }

    let latest_link_nodes = document
        .find(Name("a"))
        .filter_map(|node| {
            let href = node.attr("href");
            if href.is_none() {
                return Some(Err(ScrapperError::NoHrefAttribute));
            }
            if href?.contains(&"/latest/") {
                Some(Ok(node))
            } else {
                None
            }
        })
        .collect::<Result<Vec<Node>, ScrapperError>>()?;

    if latest_link_nodes.is_empty() {
        return Err(ScrapperError::NotFoundAnyLinks);
    }

    let links: Vec<String> = latest_link_nodes
        .iter()
        .map(|node| format!("{}{}", GITHUB_URL, node.attr("href").unwrap()))
        .collect();

    Ok((links, repo_version.to_owned()))
}

use select::document::Document;
use select::node::Node;
use select::predicate::Name;

use crate::scrapper::ScrapperError::ScrapperError;

const GITHUB_URL: &str = "https://github.com";

pub fn scrape_latest_data(html: String) -> Result<(Vec<String>, Vec<String>), ScrapperError> {
    let what = html.as_str();
    let next_what = Document::from(what);
    let latest_link_nodes = next_what
        .find(Name("a"))
        .filter_map(|node| {
            let href = node.attr("href");
            if let None = href {
                return Some(Err(ScrapperError::NoHrefAttribute));
            }
            if href.unwrap().contains(&"/latest/") {
                return Some(Ok(node));
            } else {
                return None;
            }
        })
        .collect::<Result<Vec<Node>, ScrapperError>>()?;

    if latest_link_nodes.is_empty() {
        return Err(ScrapperError::NotFoundAnyLinks);
    }

    let date_time_list = latest_link_nodes
        .iter()
        .map(|node| -> Result<String, ScrapperError> {
            Ok(node
                .parent()
                .ok_or(ScrapperError::NoParentNode)?
                .parent()
                .ok_or(ScrapperError::NoParentNode)?
                .find(Name("relative-time"))
                .collect::<Vec<Node>>()
                .first()
                .ok_or(ScrapperError::RelativeTimeNotFound)?
                .attr("datetime")
                .ok_or(ScrapperError::DateTimeNotFound)?)
            .map(|url| url.to_string())
        })
        .collect::<Result<Vec<String>, ScrapperError>>()?;

    let links: Vec<String> = latest_link_nodes
        .iter()
        .map(|node| format!("{}{}", GITHUB_URL, node.attr("href").unwrap()))
        .collect();

    return Ok((links, date_time_list));
}

#[test]
fn it_correctly_scrape_data() {
    use crate::tests::data::{LINKS, RESOURCE_TIMESTAMP};

    let file_content = std::fs::read_to_string("./src/tests/releases_nightly.htm").unwrap();

    let (scraped_links, timestamps) = match scrape_latest_data(file_content) {
        Ok((scraped_links, timestamps)) => (scraped_links, timestamps),
        Err(err) => panic!("{}", err),
    };
    assert_eq!(LINKS, scraped_links[..]);
    assert_eq!(RESOURCE_TIMESTAMP, timestamps[..]);
}

use std::io;

use select::document::Document;
use select::node::Node;
use select::predicate::Name;
use std::fmt;
use std::error;

#[derive(Debug)]
pub enum MyError {
    CannotOpenFile(io::Error),
    CannotReadData(io::Error),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}
impl error::Error for MyError {}


const GITHUB_URL: &str = "https://github.com";

pub fn scrape_latest_data(html: String) -> Option<(Vec<String>, Vec<String>)> {
    let what = html.as_str();
    let next_what = Document::from(what);
    let latest_link_nodes = next_what
        .find(Name("a"))
        .filter(|node| {
            node.attr("href")
                .expect("Not found any links on page!")
                .contains(&"/latest/")
        })
        .collect::<Vec<Node>>();

    if latest_link_nodes.is_empty() {
        panic!("No links that contains /latest/ string found on page!");
    }

    let date_time_list = latest_link_nodes
        .iter()
        .map(|node| -> Result<String, MyError> {
            Ok(node.parent()?
                .parent()?
                .find(Name("relative-time"))
                .collect::<Vec<Node>>()
                .first()?
                .attr("datetime")
                .map(|url| url.to_string())?)
        })
        .collect::<Result<Vec<String>, MyError>>();

    let links: Option<Vec<String>> = latest_link_nodes
        .iter()
        .map(|node| -> Option<String> { Some(format!("{}{}", GITHUB_URL, node.attr("href")?)) })
        .collect();

    return Some((links?, date_time_list?));
}

#[test]
fn it_correctly_scrape_data() {
    use crate::tests::data::{LINKS, RESOURCE_TIMESTAMP};

    let file_content = std::fs::read_to_string("./src/tests/releases_nightly.htm").unwrap();

    let (scraped_links, timestamps) = match scrape_latest_data(file_content) {
        Some((scraped_links, timestamps)) => (scraped_links, timestamps),
        None =>  {
            println!("what?");
            return ()
        },
    }; 
    assert_eq!(LINKS, scraped_links[..]);
    assert_eq!(RESOURCE_TIMESTAMP, timestamps[..]);
}

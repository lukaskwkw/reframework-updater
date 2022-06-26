use select::document::Document;
use select::node::Node;
use select::predicate::Name;

const GITHUB_URL: &str = "https://github.com";

pub fn scrape_latest_data(html: String) -> (Vec<String>, Vec<String>) {
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
        .map(|node| {
            node.parent()
                .unwrap()
                .parent()
                .unwrap()
                .find(Name("relative-time"))
                .collect::<Vec<Node>>()
                .first()
                .expect("Not found <relative-time [...] /> DOM node")
                .attr("datetime")
                .map(|url| url.to_string())
                .expect("No datetime attribute found in <relative-time [...] /> DOM node")
        })
        .collect::<Vec<String>>();

    let links: Vec<String> = latest_link_nodes
        .iter()
        .map(|node| format!("{}{}", GITHUB_URL, node.attr("href").unwrap()))
        .collect();

    return (links, date_time_list);
}

#[test]
fn it_correctly_scrape_data() {
    use crate::tests::data::{LINKS, RESOURCE_TIMESTAMP};

    let file_content = std::fs::read_to_string("./src/releases_nightly.htm").unwrap();

    let (scraped_links, timestamps) = scrape_latest_data(file_content);
    assert_eq!(LINKS, scraped_links[..]);
    assert_eq!(RESOURCE_TIMESTAMP, timestamps[..]);
}

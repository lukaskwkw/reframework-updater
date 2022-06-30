use crate::scrapper::scrapper::scrape_latest_data;

use crate::scrapper::ScrapperError::ScrapperError;

#[test]
fn it_correctly_scrape_data() {
    use crate::tests::data::LINKS;

    let file_content = std::fs::read_to_string("./src/tests/releases_nightly.htm").unwrap();
    let local_version = "v1.165-d96992f";

    let (scraped_links, repo_version) = match scrape_latest_data(file_content, local_version) {
        Ok((scraped_links, repo_version)) => (scraped_links, repo_version),
        Err(err) => panic!("{}", err),
    };
    assert_eq!(LINKS, scraped_links[..]);
    assert_eq!("v1.167-d96992f", repo_version);
}

#[test]
fn it_correctly_scrape_data_when_only_hash_provided() {
    use crate::tests::data::LINKS;

    let file_content = std::fs::read_to_string("./src/tests/releases_nightly.htm").unwrap();
    let local_version = "a96992f";

    let (scraped_links, repo_version) = match scrape_latest_data(file_content, local_version) {
        Ok((scraped_links, repo_version)) => (scraped_links, repo_version),
        Err(err) => panic!("{}", err),
    };
    assert_eq!(LINKS, scraped_links[..]);
    assert_eq!("v1.167-d96992f", repo_version);
}

#[test]
fn it_correctly_scrape_data_when_only_hash_differ() {
    use crate::tests::data::LINKS;

    let file_content = std::fs::read_to_string("./src/tests/releases_nightly.htm").unwrap();
    let local_version = "v1.167-a96xxcf";

    let (scraped_links, repo_version) = match scrape_latest_data(file_content, local_version) {
        Ok((scraped_links, repo_version)) => (scraped_links, repo_version),
        Err(err) => panic!("{}", err),
    };
    assert_eq!(LINKS, scraped_links[..]);
    assert_eq!("v1.167-d96992f", repo_version);
}

#[test]
fn should_throw_nothing_new_err() {
    use crate::tests::data::LINKS;

    let file_content = std::fs::read_to_string("./src/tests/releases_nightly.htm").unwrap();
    let local_version = "v1.167-d96992f";

    let (scraped_links, repo_version) = match scrape_latest_data(file_content, local_version) {
        Ok((scraped_links, repo_version)) => (scraped_links, repo_version),
        Err(err) => {
            assert_eq!(ScrapperError::NothingToUpdate, err);
            return ();
        }
    };
}

#[test]
fn should_throw_nothing_new_err_hash() {
    use crate::tests::data::LINKS;

    let file_content = std::fs::read_to_string("./src/tests/releases_nightly.htm").unwrap();
    let local_version = "d96992f";

    let (scraped_links, repo_version) = match scrape_latest_data(file_content, local_version) {
        Ok((scraped_links, repo_version)) => (scraped_links, repo_version),
        Err(err) => {
            assert_eq!(ScrapperError::NothingToUpdate, err);
            return ();
        }
    };
}

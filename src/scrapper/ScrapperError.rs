use std::error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ScrapperError {
    NotFoundAnyLinks,
    RelativeTimeNotFound,
    DateTimeNotFound,
    NoHrefAttribute,
    NoParentNode,
    NothingToUpdate,
    GetRepoVersion,
    VersionParserError,
}

impl fmt::Display for ScrapperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ScrapperError::*;
        match self {
            RelativeTimeNotFound => write!(f, "Not found <relative-time [...] /> DOM node"),
            DateTimeNotFound => write!(
                f,
                "No datetime attribute found in <relative-time [...] /> DOM node"
            ),
            NotFoundAnyLinks => write!(f, "Not found any links on page!"),
            NoHrefAttribute => write!(f, "No href attribute found"),
            NoParentNode => write!(f, "No parent node found"),
            NothingToUpdate => write!(f, "No new version found"),
            GetRepoVersion => write!(f, "Couldn't retrieve repo version"),
            VersionParserError => write!(f, "Something went wrong when parsing version"),
        }
    }
}
impl error::Error for ScrapperError {}

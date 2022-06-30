use std::io::{Cursor};
use std::{
    error::Error,
};

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

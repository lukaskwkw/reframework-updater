use std::path::Path;

use crate::{DynResult, VerResult};

const VERSION_FILENAME: &str = "reframework_revision.txt";

pub fn getLocalVersions(game_paths: &Vec<impl AsRef<Path>>) -> DynResult<Vec<VerResult>> {
    Ok(game_paths
        .iter()
        .map(|p| mapToVersion(p))
        .collect::<Vec<_>>())
}

fn mapToVersion(path: impl AsRef<Path>) -> VerResult {
    let version_file = path.as_ref().join(VERSION_FILENAME);
    println!("version_file {:?}", version_file);
    let version = std::fs::read_to_string(version_file)?;
    return Ok(version[..7].to_string());
}

use log::trace;
use self_update::update::Release;

use crate::reframework_github::refr_github::REFRGithubError;
use crate::DynResult;

use std::path::PathBuf;

use std::path::Path;

pub fn get_local_path_to_cache_folder(
    release: Option<&Release>,
    ver: Option<&str>,
) -> DynResult<PathBuf> {
    trace!("release {:#?} ver {:?}", release, ver);
    let version: String;
    if ver.is_none() {
        version = match release {
            Some(it) => it.name.to_string(),
            None => {
                return Err(Box::new(
                    REFRGithubError::VersionIsNoneAndReleaseIsNone,
                ))
            }
        }
    } else {
        version = ver.unwrap().to_string();
    };

    let path = format!("refr_cache/{}", version);
    let path = Path::new(&path);
    let folders = Path::new(path);
    let mut path_buff = PathBuf::new();
    path_buff.push(folders);
    return Ok(path_buff);
}

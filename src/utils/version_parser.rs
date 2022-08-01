use std::num::ParseIntError;

use simple_log::log::{debug, warn};
const VERSION_DELIMITER: char = '.';
pub const HASH_DELIMITER: char = '-';

fn get_version_tuple(repo: &str) -> Option<(&str, &str, &str)> {
    let (major, minor_a_hash) = repo[1..].split_once(VERSION_DELIMITER)?;
    let (minor, hash) = minor_a_hash.split_once(HASH_DELIMITER)?;
    Some((major, minor, hash))
}

pub fn isRepoVersionNewer(local: &str, repo: &str) -> Option<bool> {
    let repo_version = get_version_tuple(repo).or_else(|| {
        warn!("Repo version parser error: {}", repo);
        None
    })?;
    let local_version: (&str, &str, &str) = match local.contains('.') {
        false => {
            debug!("Local version has only hash");
            if repo_version.2 != local {
                debug!("and repo version is newer");
                return Some(true);
            }
            debug!("and Update is not required");
            return Some(false);
        }
        true => get_version_tuple(local)?,
    };
    debug!("{:?}", local_version);
    debug!("{:?}", repo_version);

    let map_err = |err: ParseIntError| -> ParseIntError {
        warn!("{:?}", err);
        err
    };

    if repo_version.0.parse::<u16>().map_err(map_err).ok()?
        > local_version.0.parse::<u16>().map_err(map_err).ok()?
    {
        debug!("Major is greater");
        Some(true)
    } else if repo_version.1.parse::<u16>().map_err(map_err).ok()?
        > local_version.1.parse::<u16>().map_err(map_err).ok()?
    {
        debug!("Minor is greater");
        Some(true)
    } else if repo_version.2 != local_version.2 {
        debug!("Major and Minor are same but Hashes are different - treating like github repo is newer");
        Some(true)
    } else {
        debug!("Update is not required");
        Some(false)
    }
}

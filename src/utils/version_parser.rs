const VERSION_DELIMETER: char = '.';
const HASH_DELIMETERE: char = '-';

fn get_version_tuple(text: &str) -> Option<(&str, &str, &str)> {
    let (major, minor_a_hash) = text[1..].split_once(VERSION_DELIMETER)?;
    let (minor, hash) = minor_a_hash.split_once(HASH_DELIMETERE)?;
    Some((major, minor, hash))
}

pub fn isRepoVersionNewer(local: &str, repo: &str) -> Option<bool> {
    let repo_version = get_version_tuple(repo)?;
    let local_version: (&str, &str, &str) = match local.contains('.') {
        false => {
            if repo_version.2 != local {
                return Some(true);
            }
            return Some(false);
        }
        true => get_version_tuple(local)?,
    };

    if repo_version.0.parse::<u8>().ok()? > local_version.0.parse().ok()? {
        Some(true)
    } else if repo_version.1.parse::<u8>().ok()? > local_version.1.parse().ok()? {
        Some(true)
    } else if repo_version.2 != local_version.2 {
        Some(true)
    } else {
        Some(false)
    }
}

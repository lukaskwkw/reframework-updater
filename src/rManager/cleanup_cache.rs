use std::fs;

use std::path::Path;

use crate::rManager::rManager_header::REvilManagerError;
use crate::utils::get_local_path_to_cache::get_local_path_to_cache_folder;

use error_stack::{IntoReport, Report, ResultExt};

use log::debug;

use crate::rManager::rManager_header::ResultManagerErr;

pub fn cleanup_cache(
    last_ver: &Vec<String>,
    game_short_name: &str,
) -> ResultManagerErr<()> {
    if last_ver.len() < 2 {
        debug!(
            "A Game {} Cache warn: {:?}",
            game_short_name,
            REvilManagerError::CacheNotFoundForGivenVersion(last_ver[0].to_string()).to_string()
        );
        return Ok(());
    }
    let last_ver_nb = &last_ver[0];
    let cache_dir = get_local_path_to_cache_folder(None, Some(last_ver_nb)).map_err(|_| 
        Report::new(REvilManagerError::ReleaseManagerIsNotInitialized,
    ))?;
    if cache_dir.exists() {
        let file_to_remove = cache_dir.join(&last_ver[1]);
        if Path::new(&file_to_remove).exists() {
            fs::remove_file(&file_to_remove).report().change_context(
                REvilManagerError::RemoveZipAssetFromCacheErr(file_to_remove.display().to_string()),
            )?;
        }
        match fs::remove_dir(&cache_dir) {
            Ok(()) => debug!("Directory: {} Removed", cache_dir.display().to_string()),
            Err(err) => debug!(
                "Can not Remove directory: {} Err {}",
                cache_dir.display().to_string(),
                err
            ),
        };
    };
    Ok(())
}

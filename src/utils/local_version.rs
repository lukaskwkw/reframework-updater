use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    path::Path,
};

use crate::{tomlConf::configStruct::Runtime, GAMES_NEXTGEN_SUPPORT, STANDARD_TYPE_QUALIFIER};
use error_stack::{IntoReport, Result, ResultExt};
use log::{debug, warn};

use super::binSearch::find_string_in_binary_file;

const VERSION_FILENAME: &str = "reframework_revision.txt";

pub struct LocalProvider;

pub struct LocalGameConfig {
    pub version: Option<String>,
    pub runtime: Option<Runtime>,
    pub nextgen: Option<bool>,
}

pub trait LocalFiles {
    fn get_local_report_for_game(&self, game_path: &str, game_short_name: &str) -> LocalGameConfig;
}

pub fn create_tdb_string(game_short_name: &str) -> String {
    format!("{}{}", game_short_name, STANDARD_TYPE_QUALIFIER)
}

#[derive(Debug)]
pub struct LocalError;

impl Display for LocalError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        fmt.write_str("Local error")
    }
}

impl Error for LocalError {}

type LocalResult<T> = Result<T, LocalError>;

impl LocalFiles for LocalProvider {
    fn get_local_report_for_game(&self, game_path: &str, game_short_name: &str) -> LocalGameConfig {
        LocalGameConfig {
            runtime: map_to_runtime(game_path),
            version: map_to_version(game_path),
            nextgen: map_to_nextgen(game_path, game_short_name),
        }
    }
}

fn map_to_nextgen(path: impl AsRef<Path>, game_short_name: &str) -> Option<bool> {
    let dinput8_path = path.as_ref().join("dinput8.dll");

    if GAMES_NEXTGEN_SUPPORT.contains(&game_short_name) {
        let text = create_tdb_string(game_short_name);
        let is_standard_edition = match find_string_in_binary_file(&dinput8_path, &text) {
            Ok(it) => it,
            Err(err) => {
                warn!(
                    "Reading binary file {} failed: {:?}",
                    dinput8_path.display(),
                    err
                );
                return None;
            }
        };

        return Some(!is_standard_edition);
    }
    None
}

fn map_to_runtime(path: impl AsRef<Path>) -> Option<Runtime> {
    let open_vr_runtime = path.as_ref().join::<String>(Runtime::OpenVR.as_local_dll());
    if Path::new(&open_vr_runtime).exists() {
        return Some(Runtime::OpenVR);
    }
    let open_xr_runtime = path.as_ref().join(Runtime::OpenXR.as_local_dll());
    if Path::new(&open_xr_runtime).exists() {
        return Some(Runtime::OpenXR);
    }
    debug!("open_vr_runtime not found {:?}", open_vr_runtime);
    debug!("open_xr_runtime not found {:?}", open_xr_runtime);
    None
}

fn map_to_version(path: impl AsRef<Path>) -> Option<String> {
    let version_file = path.as_ref().join(VERSION_FILENAME);
    let version = match std::fs::read_to_string(&version_file)
        .report()
        .change_context(LocalError)
        .attach_printable_lazy(|| {
            format!(
                "Could not read version from file {}",
                version_file.display()
            )
        }) {
        Ok(it) => it,
        Err(err) => {
            warn!("{:?}", err);
            return None;
        }
    };
    if version.len() < 7 {
        warn!(
            "version {:?} in file {} might be corrupted - version.len is lower than 7 chars",
            version,
            version_file.display()
        );
        return None;
    }
    Some(version[..7].to_string())
}

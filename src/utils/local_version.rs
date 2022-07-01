use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    path::Path,
};

use crate::tomlConf::configStruct::Runtime;
use error_stack::{IntoReport, Report, Result, ResultExt};

const VERSION_FILENAME: &str = "reframework_revision.txt";

pub struct LocalProvider;
pub type VerResult = LocalResult<Option<String>>;

pub struct LocalGameConfig {
    pub version: Option<String>,
    pub runtime: Option<Runtime>,
}

pub trait LocalFiles {
    fn get_local_report_for_game(&self, game_path: &str) -> LocalResult<LocalGameConfig>;
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
    fn get_local_report_for_game(&self, game_path: &str) -> LocalResult<LocalGameConfig> {
        return Ok(LocalGameConfig {
            runtime: map_to_runtime(game_path),
            version: map_to_version(game_path)?,
        });
    }
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
    #[cfg(debug_assertions)]
    println!("open_vr_runtime not found {:?}", open_vr_runtime);
    #[cfg(debug_assertions)]
    println!("open_xr_runtime not found {:?}", open_xr_runtime);
    None
}

fn map_to_version(path: impl AsRef<Path>) -> VerResult {
    let version_file = path.as_ref().join(VERSION_FILENAME);
    // println!("version_file {:?}", version_file);
    let version = std::fs::read_to_string(&version_file)
        .report()
        .change_context(LocalError)
        .attach_printable_lazy(|| format!("Could not read version from file {}", version_file.display()))?;
    return Ok(Some(version[..7].to_string()));
}

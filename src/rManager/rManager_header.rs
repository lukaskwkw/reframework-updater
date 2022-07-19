use std::{
    error::Error,
    ffi::OsStr,
    fmt::{self, format},
};

use log::Level;
use self_update::update::ReleaseAsset;

use crate::{
    refr_github::{ManageGithub, REFRGithub},
    steam::SteamThings,
    tomlConf::{
        config::ConfigProvider,
        configStruct::{GameConfig, REvilConfig, SteamId, ShortGameName},
    },
    utils::local_version::LocalFiles,
    DynResult, dialogs::{dialogs::{Ask}, dialogs_label::LabelOptions}, args::RunAfter,
};
use error_stack::Result;

pub type ResultManagerErr<T> = Result<T, REvilManagerError>;

pub const SORT_DETERMINER: &str = "info";
pub struct REvilManager {
    pub config: REvilConfig,
    pub config_provider: Box<dyn ConfigProvider>,
    pub steam_menago: Box<dyn SteamThings>,
    pub local_provider: Box<dyn LocalFiles>,
    pub dialogs: Box<dyn Ask>,
    pub github_release_manager: Option<Box<dyn ManageGithub>>,
    pub refr_ctor: fn(&str, &str) -> Box<dyn ManageGithub>,
    pub state: REvilManagerState,
}

pub trait REvilThings {
    fn load_config(&mut self) -> ResultManagerErr<&mut Self>;
    fn attach_logger(&mut self) -> ResultManagerErr<&mut Self>;
    fn load_games_from_steam(&mut self) -> ResultManagerErr<&mut Self>;
    fn generate_main_defaults(&mut self) -> Result<&mut Self, REvilManagerError>;
    fn get_local_settings_per_game(&mut self) -> &mut Self;
    fn generate_ms_links(&mut self) -> Result<&mut Self, REvilManagerError>;
    fn check_for_REFramework_update(&mut self) -> ResultManagerErr<&mut Self>;
    fn pick_one_game_from_report(&mut self) -> ResultManagerErr<&mut Self>;
    fn ask_for_decision(&mut self) -> ResultManagerErr<&mut Self>;
    fn download_REFramework_update(&mut self) -> ResultManagerErr<&mut Self>;
    fn unzip_update<F: Fn(&OsStr) -> bool>(
        &self,
        game_short_name: &str,
        file_name: &str,
        version: Option<&str>,
        unzip_skip_fun: Option<F>,
    ) -> ResultManagerErr<&Self>
    where
        F: Fn(&OsStr) -> bool;
    fn unzip_updates(&mut self) -> &mut Self;
    fn after_unzip_work(&mut self) -> Result<&mut Self, REvilManagerError>;
    fn save_config(&mut self) -> ResultManagerErr<&mut Self>;
    fn ask_for_game_decision_if_needed(&mut self) -> ResultManagerErr<&mut Self>;
    fn ask_for_switch_type_decision(&mut self, run_after: RunAfter) -> ResultManagerErr<&mut Self>;
    fn load_from_cache_if_chosen(&mut self) -> ResultManagerErr<&mut Self>;
    fn check_for_self_update(&mut self) -> DynResult<&mut Self>;
    fn self_update(&mut self) -> DynResult<&mut Self>;
    fn before_launch_procedure(&self, steam_id: &String) -> ResultManagerErr<()>;
    fn launch_game(&mut self) -> ResultManagerErr<&mut Self>;
    fn bind(
        &mut self,
        cb: impl Fn(&mut REvilManager) -> ResultManagerErr<&mut Self>,
        log_level: Level,
    ) -> &mut Self;
    fn or_log_err(
        &mut self,
        cb: impl Fn(&mut REvilManager) -> ResultManagerErr<&mut Self>,
        log_level: Level,
    ) -> &mut Self;
}

#[derive(Debug, Default)]
pub enum REvilManagerError {
    ReleaseIsEmpty,
    CheckingNewReleaseErr,
    GameNotFoundForGivenShortName(String),
    GameNotFoundForGivenSteamId(String),
    CannotDeductShortNameFromAssetName(String),
    RemoveFileFailed(String),
    RemoveZipAssetFromCacheErr(String),
    CacheNotFoundForGivenVersion(String),
    FailedToCreateMsLink(String),
    ReleaseManagerIsNotInitialized,
    GameLocationMissing,
    ModRuntimeIsNone(String),
    GetLocalPathToCacheErr,
    UnzipError,
    SaveConfigError,
    LoadConfigError,
    #[default]
    Other,
}

impl fmt::Display for REvilManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            REvilManagerError::ReleaseIsEmpty => write!(f, "ReleaseIsEmpty"),
            REvilManagerError::CheckingNewReleaseErr => write!(f, "CheckingNewReleaseErr"),
            REvilManagerError::GameNotFoundForGivenShortName(info) => {
                write!(f, "GameNotFoundForGivenShortName {}", info)
            }
            REvilManagerError::CannotDeductShortNameFromAssetName(info) => {
                write!(f, "CannotDeductShortNameFromAssetName {}", info)
            }
            REvilManagerError::RemoveZipAssetFromCacheErr(info) => {
                write!(f, "RemoveZipAssetFromCacheErr {}", info)
            }
            REvilManagerError::ReleaseManagerIsNotInitialized => {
                write!(f, "ReleaseManagerIsNotInitialized")
            }
            REvilManagerError::GameLocationMissing => write!(f, "GameLocationMissing"),
            REvilManagerError::UnzipError => write!(f, "UnzipError"),
            REvilManagerError::SaveConfigError => write!(f, "SaveConfigError"),
            REvilManagerError::LoadConfigError => write!(f, "LoadConfigError"),
            REvilManagerError::Other => write!(f, "Other"),
            REvilManagerError::RemoveFileFailed(info) => write!(f, "RemoveFileFiled {}", info),
            REvilManagerError::GameNotFoundForGivenSteamId(info) => {
                write!(f, "GameNotFoundForGivenSteamId {}", info)
            }
            REvilManagerError::CacheNotFoundForGivenVersion(info) => {
                write!(f, "CacheNotFoundForGivenVersion {}", info)
            }
            REvilManagerError::FailedToCreateMsLink(info) => {
                write!(f, "FailedToCreateMsLink {}", info)
            }
            REvilManagerError::GetLocalPathToCacheErr => write!(f, "GetLocalPathToCacheErr"),
            REvilManagerError::ModRuntimeIsNone(game) => write!(f, "ModRuntimeIsNone for {}", game),
        }
    }
}

impl Error for REvilManagerError {}
#[derive(Default)]
pub struct REvilManagerState {
    pub skip_next: bool,
    pub games_that_require_update: Vec<ShortGameName>,
    pub selected_assets: Vec<ReleaseAsset>,
    pub selected_game_to_launch: Option<SteamId>,
    pub config_loading_error_ocurred: bool,
    pub new_steam_game_found: bool,
    pub selected_option: Option<LabelOptions>,
}
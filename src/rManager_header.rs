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
        configStruct::{GameConfig, REvilConfig, SteamId},
    },
    utils::local_version::LocalFiles,
    DynResult,
};
use error_stack::Result;

#[derive(Debug, Default)]
pub enum REvilManagerError {
    ReleaseIsEmpty,
    CheckingNewReleaseErr,
    GameNotFoundForGivenShortName(String),
    GameNotFoundForGivenSteamId(String),
    CannotDeductShortNameFromAssetName(String),
    RemoveFileFiled(String),
    RemoveZipAssetFromCacheErr(String),
    CacheNotFoundForGivenVersion(String),
    FailedToCreateMsLink(String),
    ReleaseManagerIsNotInitialized,
    GameLocationMissing,
    UnzipError,
    SaveConfigError,
    GameToLaunchIsNone,
    ReadDirError(String),
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
            REvilManagerError::GameToLaunchIsNone => write!(f, "GameToLaunchIsNone"),
            REvilManagerError::ReadDirError(info) => write!(f, "ReadDirError {}", info),
            REvilManagerError::LoadConfigError => write!(f, "LoadConfigError"),
            REvilManagerError::Other => write!(f, "Other"),
            REvilManagerError::RemoveFileFiled(info) => write!(f, "RemoveFileFiled {}", info),
            REvilManagerError::GameNotFoundForGivenSteamId(info) => {
                write!(f, "GameNotFoundForGivenSteamId {}", info)
            }
            REvilManagerError::CacheNotFoundForGivenVersion(info) => {
                write!(f, "CacheNotFoundForGivenVersion {}", info)
            }
            REvilManagerError::FailedToCreateMsLink(info) => {
                write!(f, "FailedToCreateMsLink {}", info)
            }
        }
    }
}

impl Error for REvilManagerError {}

#[derive(Debug, Default, PartialEq)]
pub enum LabelOptions {
    SwitchType,
    LoadDifferentVersionFromCache,
    Skip,
    Exit,
    UpdateAllGames,
    UpdateAllGamesAutoDetect,
    UpdateAllGamesPreferStandard,
    UpdateAllGamesPreferNextgen,
    #[default]
    Other,
}

impl From<&str> for LabelOptions {
    fn from(text: &str) -> Self {
        match text {
            "Switch type..." => LabelOptions::SwitchType,
            "Load from cache..." => LabelOptions::LoadDifferentVersionFromCache,
            "Skip" => LabelOptions::Skip,
            "Exit" => LabelOptions::Exit,
            "Update all games" => LabelOptions::UpdateAllGames,
            "Update all games - prefer standard" => LabelOptions::UpdateAllGamesPreferStandard,
            "Update all games - prefer nextgen" => LabelOptions::UpdateAllGamesPreferNextgen,
            "Update all games - autodetect" => LabelOptions::UpdateAllGamesAutoDetect,
            _ => LabelOptions::Other,
        }
    }
}

impl LabelOptions {
    pub fn to_label(&self) -> String {
        match self {
            LabelOptions::SwitchType => "Switch type...".to_string(),
            LabelOptions::LoadDifferentVersionFromCache => "Load from cache...".to_string(),
            LabelOptions::Skip => "Skip".to_string(),
            LabelOptions::Exit => "Exit".to_string(),
            LabelOptions::UpdateAllGames => "Update all games".to_string(),
            LabelOptions::UpdateAllGamesPreferStandard => "Update all games - prefer standard".to_string(),
            LabelOptions::UpdateAllGamesPreferNextgen => "Update all games - prefer nextgen".to_string(),
            LabelOptions::UpdateAllGamesAutoDetect => "Update all games - autodetect".to_string(),
            other => format!("{:?}", other),
        }
    }
}

#[derive(Default)]
pub struct REvilManagerState {
    pub skip_next: bool,
    pub games_that_require_update: Vec<String>,
    pub selected_assets: Vec<ReleaseAsset>,
    pub selected_game_to_launch: Option<SteamId>,
    pub config_loading_error_ocurred: bool,
    pub new_steam_game_found: bool,
    pub other_option: Option<LabelOptions>,
}

pub struct REvilManager {
    pub config: REvilConfig,
    pub config_provider: Box<dyn ConfigProvider>,
    pub steam_menago: Box<dyn SteamThings>,
    pub local_provider: Box<dyn LocalFiles>,
    pub github_release_manager: Option<Box<dyn ManageGithub>>,
    pub refr_ctor: fn(&str, &str) -> REFRGithub,
    pub state: REvilManagerState,
}

pub type ResultManagerErr<T> = Result<T, REvilManagerError>;

pub const SORT_DETERMINER: &str = "info";

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
    fn unzip_updates(&self) -> ResultManagerErr<&Self>;
    fn after_unzip_work(&mut self) -> Result<&mut Self, REvilManagerError>;
    fn save_config(&mut self) -> ResultManagerErr<&mut Self>;
    fn ask_for_game_decision_if_needed(&mut self) -> ResultManagerErr<&mut Self>;
    fn ask_for_switch_type_decision(&mut self) -> ResultManagerErr<&mut Self>;
    fn check_for_self_update(&mut self) -> DynResult<&mut Self>;
    fn self_update(&mut self) -> DynResult<&mut Self>;
    fn before_launch_procedure(&self, steam_id: &String) -> ResultManagerErr<()>;
    fn launch_game(&mut self) -> ResultManagerErr<&mut Self>;
    fn find_game_conf_by_steam_id(
        &self,
        steam_id: &String,
    ) -> ResultManagerErr<(&String, &GameConfig)>;
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

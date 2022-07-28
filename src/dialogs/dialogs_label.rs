type Version = String;
type AssetName = String;

#[derive(Debug, Default, PartialEq)]
pub enum LabelOptions {
    SwitchType,
    SwitchToStandard(String),
    SwitchToNextgen(String),
    SwitchRuntime,
    LoadDifferentVersionFromCache,
    LoadFromCache(ShortGameName, AssetName, Version),
    Skip,
    Back,
    GoTop,
    Exit,
    UpdateAllGames,
    UpdateAllGamesAutoDetect,
    UpdateAllGamesPreferStandard,
    UpdateAllGamesPreferNextgen,
    #[default]
    Other,
}
use LabelOptions::*;

use crate::tomlConf::configStruct::ShortGameName;
pub static SWITCH_RUNTIME_PART: &str = "Switch runtime to";

impl From<&str> for LabelOptions {
    fn from(text: &str) -> Self {
        match text {
            "Switch type..." => SwitchType,
            "Load from cache..." => LoadDifferentVersionFromCache,
            "Skip" => Skip,
            "Exit" => Exit,
            "Update all games" => UpdateAllGames,
            "Update all games - prefer standard" => UpdateAllGamesPreferStandard,
            "Update all games - prefer nextgen" => UpdateAllGamesPreferNextgen,
            "Update all games - autodetect" => UpdateAllGamesAutoDetect,
            "Back" => Back,
            "Back to download section" => GoTop,
 
            label => deduct_switch_to(label)
                .or_else(|| deduct_load_from_cache(label))
                .or_else(|| label.contains(SWITCH_RUNTIME_PART).then_some(SwitchRuntime))
                .unwrap_or(Other),
        }
    }
}

fn deduct_load_from_cache(label: &str) -> Option<LabelOptions> {
    label
        .contains("from cache")
        .then(|| match label.splitn(4, '|').collect::<Vec<&str>>()[..] {
            [_, short_name, asset_name, version] => LoadFromCache(
                short_name.to_string(),
                asset_name.to_string(),
                version.to_string(),
            ),
            _ => Other,
        })
}

fn deduct_switch_to(label: &str) -> Option<LabelOptions> {
    label
        .contains("Switch type to |")
        .then(|| {
            let (_, game_type) = label.split_once('|')?;
            label.split_once(" - ").map(|(_, short_name)| {
                if game_type == "standard" {
                    SwitchToStandard(short_name.to_string())
                } else {
                    SwitchToNextgen(short_name.to_string())
                }
            })
        })
        .unwrap_or(None)
}

impl LabelOptions {
    pub fn to_label(&self) -> String {
        match self {
            SwitchType => "Switch type...".to_string(),
            LoadDifferentVersionFromCache => "Load from cache...".to_string(),
            Skip => "Skip".to_string(),
            Exit => "Exit".to_string(),
            UpdateAllGames => "Update all games".to_string(),
            UpdateAllGamesPreferStandard => "Update all games - prefer standard".to_string(),
            UpdateAllGamesPreferNextgen => "Update all games - prefer nextgen".to_string(),
            UpdateAllGamesAutoDetect => "Update all games - autodetect".to_string(),
            SwitchToStandard(game_short_name) => {
                format!("Switch type to |standard| - {}", game_short_name)
            }
            SwitchToNextgen(game_short_name) => {
                format!("Switch type to |nextgen| - {}", game_short_name)
            }
            SwitchRuntime => "SwitchRuntime".to_string(),
            LoadFromCache(short_name, asset_name, version) => format!(
                "Load mod from cache |{}|{}|{}",
                short_name, asset_name, version
            ),
            Other => "Other".to_string(),
            Back => "Back".to_string(),
            GoTop => "Back to download section".to_string(),
        }
    }
}

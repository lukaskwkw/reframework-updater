#[derive(Debug, Default, PartialEq)]
pub enum LabelOptions {
    SwitchType,
    SwitchToStandard(String),
    SwitchToNextgen(String),
    SwitchRuntime,
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

pub static SWITCH_RUNTIME_PART: &str = "Switch runtime to";

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
            label => {
                let option = label
                    .contains("Switch type to |")
                    .then(|| {
                        let game_type = label.split('|').collect::<Vec<&str>>()[1];
                        label.split_once(" - ").map(|(_, short_name)| {
                            if game_type == "standard" {
                                LabelOptions::SwitchToStandard(short_name.to_string())
                            } else {
                                LabelOptions::SwitchToNextgen(short_name.to_string())
                            }
                        })
                    })
                    .unwrap_or(
                        label
                            .contains(SWITCH_RUNTIME_PART)
                            .then_some(LabelOptions::SwitchRuntime),
                    )
                    .unwrap_or(LabelOptions::Other);
                option
            }
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
            LabelOptions::UpdateAllGamesPreferStandard => {
                "Update all games - prefer standard".to_string()
            }
            LabelOptions::UpdateAllGamesPreferNextgen => {
                "Update all games - prefer nextgen".to_string()
            }
            LabelOptions::UpdateAllGamesAutoDetect => "Update all games - autodetect".to_string(),
            LabelOptions::SwitchToStandard(game_short_name) => {
                format!("Switch type to |standard| - {}", game_short_name)
            }
            LabelOptions::SwitchToNextgen(game_short_name) => {
                format!("Switch type to |nextgen| - {}", game_short_name)
            }
            other => format!("{:?}", other),
        }
    }
}

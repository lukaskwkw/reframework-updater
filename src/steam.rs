use core::fmt;
use error_stack::{IntoReport, Result, ResultExt};
use game_scanner::{manager, prelude::Game, steam};
use std::{
    error::Error,
    fmt::{Display, Formatter},
    path::PathBuf,
};

pub struct SteamManager;
#[derive(Debug, Default)]
pub enum SteamError {
    #[default]
    other,
    GameNotFoundById(String)
}
impl Display for SteamError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        fmt.write_str("Steam error")
    }
}

type SteamResult<T> = Result<T, SteamError>;

impl Error for SteamError {}

pub trait SteamThings {
    fn get_games_locations(&self, game_ids: &Vec<&str>) -> SteamResult<Vec<(String, PathBuf)>>;
    fn run_game(&self, game: &Game) -> SteamResult<()>;
    fn run_game_via_steam_manager(&self, id: &str) -> SteamResult<()>;
}

impl SteamThings for SteamManager {
    fn get_games_locations(&self, game_ids: &Vec<&str>) -> SteamResult<Vec<(String, PathBuf)>> {
        let games = steam::games().report().change_context(SteamError::default())?;

        let game_path_vec: Vec<(String, PathBuf)> = games
            .iter()
            .filter_map(
                |game| match game_ids.iter().any(|id| *id == game.id) {
                    true => Some((game.id.clone(), game.path.to_owned()?)),
                    false => None,
                },
            )
            .collect();
        Ok(game_path_vec)
    }

    fn run_game(&self, game: &Game) -> SteamResult<()> {
        manager::launch_game(game).report().change_context(SteamError::default())?;
        Ok(())
    }

    fn run_game_via_steam_manager(&self, id: &str) -> SteamResult<()> {
        let game = steam::find(id).report().change_context(SteamError::GameNotFoundById(id.to_string()))?;
        manager::launch_game(&game).report().change_context(SteamError::default())?;
        Ok(())
    }
}

/* TODO
     game.commands.launch = Some(vec![
        launcher_executable.display().to_string(),
        String::from("-silent"),
        format!("steam://run/{}", &game.id),
    ]);
*/
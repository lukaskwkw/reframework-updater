use crate::DynResult;
use game_scanner::{manager, prelude::Game, steam};
use std::path::PathBuf;

pub struct SteamManager;

pub trait SteamThings {
    fn get_games_locations(&self, game_ids: &Vec<&str>) -> DynResult<Vec<(String, PathBuf)>>;
    fn run_game(&self, game: &Game) -> DynResult<()>;
}

impl SteamThings for SteamManager {
    fn get_games_locations(&self, game_ids: &Vec<&str>) -> DynResult<Vec<(String, PathBuf)>> {
        let games = steam::games()?;

        let game_path_vec: Vec<(String, PathBuf)> = games
            .iter()
            .filter_map(
                |game| match game_ids.iter().any(|id| id.to_owned() == game.id) {
                    true => Some((game.id.clone(), game.path.to_owned()?)),
                    false => None,
                },
            )
            .collect();
        return Ok(game_path_vec);
    }

    fn run_game(&self, game: &Game) -> DynResult<()> {
        manager::launch_game(game)?;
        Ok(())
    }
}

use std::path::PathBuf;

use game_scanner::{manager, prelude::Game, steam};

use crate::DynResult;

pub fn get_games_locations(game_ids: Vec<&str>) -> DynResult<Vec<PathBuf>> {
    let games = steam::games()?;

    let paths: Vec<PathBuf> = games
        .iter()
        .filter_map(
            |game| match game_ids.iter().any(|id| id.to_owned() == game.id) {
                true => Some(game.path.to_owned()?),
                false => None,
            },
        )
        .collect();
    return Ok(paths);
}

pub fn run_game(game: &Game) -> DynResult<()> {
    manager::launch_game(game)?;
    Ok(())
}

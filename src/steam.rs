use std::{
    error::Error,
    path::{Path, PathBuf},
};

use game_scanner::{manager, prelude::Game, steam};

pub fn getGamesLocations(game_ids: Vec<&str>) -> Result<Vec<PathBuf>, Box<dyn Error>> {
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

pub fn runGame(game: &Game) -> Result<(), Box<dyn Error>> {
    manager::launch_game(game)?;
    Ok(())
}

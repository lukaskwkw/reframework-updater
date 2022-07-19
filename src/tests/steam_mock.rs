use std::path::Path;

use crate::steam::MockSteamThings;

pub fn prepare_steam_mock(steam_menago: &mut Box<MockSteamThings>) {
    steam_menago.expect_get_games_locations().return_once(|_| {
        let game_path_vec = [
            (
                "883710".to_string(),
                Path::new("D:/steam/games/RE2").to_path_buf(),
            ),
            (
                "418370".to_string(),
                Path::new("D:/steam/games/RE7").to_path_buf(),
            ),
            (
                "1196590".to_string(),
                Path::new("D:/steam/games/RE8").to_path_buf(),
            ),
        ]
        .to_vec();
        Ok(game_path_vec)
    });
    steam_menago
        .expect_run_game_via_steam_manager()
        .returning(|_| Ok(()));
}

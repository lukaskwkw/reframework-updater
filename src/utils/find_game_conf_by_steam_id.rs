use crate::rManager::rManager_header::REvilManagerError;

use error_stack::Report;

use crate::tomlConf::configStruct::GameConfig;

use crate::rManager::rManager_header::ResultManagerErr;

use crate::tomlConf::configStruct::REvilConfig;

pub fn find_game_conf_by_steam_id<'a>(
    config: &'a REvilConfig,
    steam_id: &'a String,
) -> ResultManagerErr<(&'a String, &'a GameConfig)> {
    let (game_short_name, game_config) = config
        .games
        .iter()
        .find(|(_, conf)| conf.steamId.as_ref().unwrap() == steam_id)
        .ok_or(Report::new(REvilManagerError::GameNotFoundForGivenSteamId(
            steam_id.to_string(),
        )))?;
    Ok((game_short_name, game_config))
}

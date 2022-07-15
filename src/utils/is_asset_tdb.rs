use crate::STANDARD_TYPE_QUALIFIER;

use crate::GAMES_NEXTGEN_SUPPORT;

use self_update::update::ReleaseAsset;

// check if asset is TDB or not if it doesn't support nextgen version then None is returned
pub fn is_asset_tdb(game_short_name: &str, asset: &ReleaseAsset) -> Option<bool> {
    if GAMES_NEXTGEN_SUPPORT.contains(&game_short_name) {
        if asset.name.contains(STANDARD_TYPE_QUALIFIER) {
            return Some(true);
        } else {
            return Some(false);
        }
    }
    None
}

use crate::{
    args::parse_args, rManager::rManager_header::REvilManager,
    rManager::rManager_header::REvilThings, ARGS,
};
use error_stack::ResultExt;
use log::Level;

pub struct StrategyFactory;

trait Strategy {
    fn run(manager: &mut REvilManager);
}

// TODO: Maybe switch bind to and_then instead

struct LaunchAndSave;
impl Strategy for LaunchAndSave {
    fn run(manager: &mut REvilManager) {
        manager.launch_game().unwrap().save_config().unwrap();
    }
}

impl StrategyFactory {
    pub fn new() -> Self {
        Self {}
    }
    pub fn get_strategy(_manager: &mut REvilManager) -> Box<fn(&mut REvilManager)> {
        let mut run = "none".to_string();
        unsafe {
            parse_args();
            if let Some(args) = &ARGS {
                run = args.one.clone();
            };
        }
        if run != "none" {
            return Box::new(CheckUpdateAndRunTheGame::run);
        } else {
            return Box::new(BindStrategy::run);
        }
    }
}

struct CheckUpdateAndRunTheGame;
impl Strategy for CheckUpdateAndRunTheGame {
    fn run(manager: &mut REvilManager) {
        EarlyLoad::run(manager);
        manager
            .bind(|this| this.check_for_REFramework_update(), Level::Error)
            .bind(|this| this.pick_one_game_from_report(), Level::Error)
            .bind(|this| this.download_REFramework_update(), Level::Error)
            .bind(
                |this| {
                    if let Err(err) = this.unzip_updates() {
                        return Err(err);
                    };
                    Ok(this)
                },
                Level::Error,
            )
            .after_unzip_work()
            .unwrap()
            .save_config()
            .unwrap()
            .ask_for_game_decision_if_needed()
            .unwrap();
        LaunchAndSave::run(manager);
    }
}

struct CheckAndRest;
impl Strategy for CheckAndRest {
    fn run(manager: &mut REvilManager) {
        manager
            .bind(|this| this.check_for_REFramework_update(), Level::Error)
            .bind(|this| this.ask_for_decision(), Level::Error)
            .bind(|this| this.download_REFramework_update(), Level::Error)
            .bind(
                |this| {
                    if let Err(err) = this.unzip_updates() {
                        return Err(err);
                    };
                    Ok(this)
                },
                Level::Error,
            )
            .after_unzip_work()
            .unwrap()
            .save_config()
            .unwrap()
            .ask_for_game_decision_if_needed()
            .unwrap()
            .ask_for_switch_type_decision()
            .unwrap();
        LaunchAndSave::run(manager);
    }
}
// struct EarlyLoad;
// impl Strategy for EarlyLoad {
//     fn run(manager: &mut REvilManager) {
//         manager
//         .load_config()
//         .attach_printable("Error loading config file.")
//         .map_or((), |xd| xd.attach_logger());
//         ()
//         // .and_then(|this|
//         //     this.load_games_from_steam()
//         //     .attach_printable("Error detecting steam games. Check generated config file and try add game manually there.")
//         // )
//         // .unwrap();
//     }
// }

struct EarlyLoad;
impl Strategy for EarlyLoad {
    fn run(manager: &mut REvilManager) {
        manager
            .or_log_err(
                |this| {
                    this.load_config()
                        .attach_printable("Error loading config file.")
                },
                Level::Warn,
            )
            .bind(
                |this| {
                    this.load_games_from_steam()
                        .attach_printable("Error detecting steam games. Check generated config file and try add game manually there.")
                },
                Level::Error,
            );
    }
}

struct BindStrategy;
impl Strategy for BindStrategy {
    fn run(manager: &mut REvilManager) {
        EarlyLoad::run(manager);
        manager
            .or_log_err(|this| this.generate_ms_links(), Level::Warn)
            .bind(
                |this| {
                    // only check against local files when a config failed to be loaded and if steam found new game
                    if this.state.config_loading_error_ocurred || this.state.new_steam_game_found {
                        return Ok(this.get_local_settings_per_game());
                    }
                    return Ok(this);
                },
                Level::Error,
            );
        CheckAndRest::run(manager);
    }
}

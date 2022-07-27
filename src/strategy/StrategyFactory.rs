use crate::{
    args::RunAfter, rManager::rManager_header::REvilManager,
    rManager::rManager_header::REvilThings, ARGS,
};
use error_stack::ResultExt;
use log::{error, info, warn, Level};

pub struct StrategyFactory;

trait Strategy {
    fn run(manager: &mut REvilManager);
}

impl StrategyFactory {
    pub fn get_strategy(_manager: &mut REvilManager) -> Box<fn(&mut REvilManager)> {
        let run = get_args();
        if run != "none" {
            Box::new(CheckUpdateAndRunTheGame::run)
        } else {
            Box::new(DefaultRoute::run)
        }
    }
}

struct CheckUpdateAndRunTheGame;
impl Strategy for CheckUpdateAndRunTheGame {
    fn run(manager: &mut REvilManager) {
        EarlyLoad::run(manager);
        manager
            .check_for_REFramework_update()
            .and_then(|this| this.pick_one_game_from_report_and_set_as_selected())
            .and_then(|this| this.download_REFramework_update())
            .unwrap()
            .unzip_updates()
            .after_unzip_work(None)
            .and_then(|this| this.save_config())
            .and_then(|this| this.ask_for_game_decision_if_needed())
            .unwrap();
        LaunchAndSave::run(manager);
    }
}

struct LaunchAndSave;
impl Strategy for LaunchAndSave {
    fn run(manager: &mut REvilManager) {
        manager
            .launch_game()
            .and_then(|this| this.save_config())
            .map(|_| ())
            .unwrap_or_else(|err| error!("{:?}", err));
    }
}

struct AskLastOptions;
impl Strategy for AskLastOptions {
    fn run(manager: &mut REvilManager) {
        manager
            .ask_for_game_decision_if_needed()
            .and_then(|this| this.ask_for_switch_type_decision(RunAfter::yes))
            .and_then(|this| this.load_from_cache_if_chosen())
            .unwrap();
    }
}

struct CheckAndRest;
impl Strategy for CheckAndRest {
    fn run(manager: &mut REvilManager) {
        manager
            .check_for_REFramework_update()
            .and_then(|this| this.ask_for_decision())
            .and_then(|this| this.download_REFramework_update())
            .unwrap()
            .unzip_updates()
            .after_unzip_work(None)
            .and_then(|this| this.save_config())
            .unwrap();
        AskLastOptions::run(manager);
        LaunchAndSave::run(manager);
    }
}

struct EarlyLoad;
impl Strategy for EarlyLoad {
    fn run(manager: &mut REvilManager) {
        manager.or_log_err(
            |this| {
                this.load_config()
                    .attach_printable("Error loading config file.")
            },
            Level::Warn,
        );
        match manager.load_games_from_steam() {
            Ok(_) => info!("Auto-detect steam games done!"),
            Err(err) => manager
                .state
                .config_loading_error_ocurred
                .then(|| panic!("Error loading config file and error steam detection: {err}. Make sure steam is installed correctly"))
                .unwrap_or(warn!("{err}")),
        };
    }
}

struct DefaultRoute;
impl Strategy for DefaultRoute {
    fn run(manager: &mut REvilManager) {
        EarlyLoad::run(manager);
        manager.or_log_err(|this| this.generate_ms_links(), Level::Warn);
        // only check local files again when a config failed to be loaded or a steam found the new game
        if manager.state.config_loading_error_ocurred || manager.state.new_steam_game_found {
            manager.get_local_settings_per_game_and_amend_current_ones();
        };

        CheckAndRest::run(manager);
    }
}

fn get_args() -> String {
    let mut run = "none".to_string();
    unsafe {
        if let Some(args) = &ARGS {
            run = args.one.clone();
        };
    }
    run
}

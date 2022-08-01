use crate::{
    dialogs::dialogs_label::LabelOptions, rManager::rManager_header::REvilManager,
    rManager::rManager_header::REvilThings, ARGS,
};
use error_stack::ResultExt;
use log::{error, info, warn, Level};

pub struct StrategyFactory;

trait Strategy {
    fn run(manager: &mut REvilManager);
}

impl StrategyFactory {
    pub fn get_strategy(manager: &mut REvilManager) -> Box<fn(&mut REvilManager)> {
        let run = get_args();

        manager.state.selected_option = Some(LabelOptions::GoTop);
        if run == "none" {
            Box::new(DefaultRoute::run)
        } else {
            Box::new(CheckUpdateAndRunTheGame::run)
        }
    }
}

struct DefaultRoute;
impl Strategy for DefaultRoute {
    fn run(manager: &mut REvilManager) {
        EarlyLoad::run(manager);
        manager.or_log_err(|this| this.generate_ms_links(), Level::Warn);
        manager
            .check_for_REFramework_update()
            .and_then(|this| this.decision_loop())
            .unwrap();
        LaunchAndSave::run(manager);
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
            .and_then(|this| this.unzip_updates().after_unzip_work(None))
            .and_then(|this| this.save_config())
            // below is only necessary when restarting program with switch/update-me option
            .and_then(|this| this.decision_loop())
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
                .then(|| {
                     error!("Error loading config file and error steam detection: {err}. Make sure steam is installed correctly");
                     panic!();
                })
                .unwrap_or(warn!("{err}")),
        };
        // only check local files again when a config failed to be loaded or a steam found the new game
        if manager.state.config_loading_error_ocurred || manager.state.new_steam_game_found {
            manager.get_local_settings_per_game_and_amend_current_ones();
        };
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

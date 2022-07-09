use crate::{
    args::parse_args,
    rManager::{REvilManager, REvilThings},
};
use error_stack::ResultExt;
use log::{debug, info, Level};

pub struct StrategyFactory;

trait Strategy {
    fn run(manager: &mut REvilManager);
}

struct ConfigFileNotFound {}

struct ConfigFileFound {}

struct NormalRoute {}

impl StrategyFactory {
    pub fn new() -> Self {
        Self {}
    }
    pub fn get_strategy(_manager: &mut REvilManager) -> impl Fn(&mut REvilManager) {
        unsafe {
            parse_args();
        }
        BindStrategy::run
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
            .launch_game()
            .unwrap();
    }
}
struct BindStrategy;
impl Strategy for BindStrategy {
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
            )
            .bind(|this|{
                // only check against local files when a config failed to be loaded
                // TODO but if steam find new game then it should also launch!
                if this.config_loading_error_ocurred { return Ok(this.get_local_settings_per_game()); }
                return Ok(this);
            }, Level::Error);
        CheckAndRest::run(manager);
    }
}

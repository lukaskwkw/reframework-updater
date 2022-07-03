use crate::rManager::{REvilManager, REvilThings};

pub struct StrategyFactory;

trait Strategy {
    fn run(manager: &mut REvilManager) -> ();
}

struct ConfigFileNotFound {}

struct ConfigFileFound {}

impl StrategyFactory {
    pub fn new() -> Self {
        Self {}
    }
    pub fn get_strategy(manager: &mut REvilManager) -> impl Fn(&mut REvilManager) -> () {
        let strategy = match manager.load_config() {
            Ok(t) => ConfigFileFound::run,
            Err(e) => {
                println!("Error loading config: {:?}", e);
                ConfigFileNotFound::run
            }
        };
        return strategy;
    }
}

impl Strategy for ConfigFileNotFound {
    fn run(manager: &mut REvilManager) {
        match manager.attach_logger().unwrap().load_games_from_steam() {
            Ok(it) => it,
            Err(err) => {
                // error!("Error loading steam games {:?}", err);
                manager.save_config().unwrap();
                println!("Consider providing own setting per game. Check generated config.toml file for more information.");
                return ();
            }
        }
        .get_local_settings_per_game()
        .check_for_REFramework_update()
        .download_REFramework_update();
        return ();
    }
}

impl Strategy for ConfigFileFound {
    fn run(manager: &mut REvilManager) {
        manager
            .attach_logger()
            .unwrap()
            .check_for_REFramework_update()
            .download_REFramework_update()
            .check_for_self_update()
            .unwrap();
        return ();
    }
}

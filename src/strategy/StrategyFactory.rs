use core::time;
use std::{process, thread};

use clap::Parser;
use error_stack::ResultExt;
use log::{error, info, warn, Level};

use crate::{
    args::{parse_args, ArgsClap},
    rManager::{REvilManager, REvilThings},
};

pub struct StrategyFactory;

trait Strategy {
    fn run(manager: &mut REvilManager) -> ();
}

struct ConfigFileNotFound {}

struct ConfigFileFound {}

struct NormalRoute {}

impl StrategyFactory {
    pub fn new() -> Self {
        Self {}
    }
    pub fn get_strategy(manager: &mut REvilManager) -> impl Fn(&mut REvilManager) -> () {
        unsafe {
            parse_args();
        }
        BindStrategy::run
    }
}

// struct AndThenStrategy;
// impl Strategy for AndThenStrategy {
//     fn run(manager: &mut REvilManager) {
//         manager
//             .or_log_err(
//                 |this| this.load_config().attach_printable("Error loading config file."),
//                 Level::Warn,
//             )
//             .load_games_from_steam()
//             .and_then(|this| this.load_games_from_steam())
//             .and_then(|this| Ok(this.get_local_settings_per_game()))
//             .and_then(|this| Ok(this.check_for_REFramework_update()))
//             .and_then(|this| Ok(this.download_REFramework_update()))
//             .unwrap();

//         ()
//     }
// }

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
            .bind(|this| Ok(this.get_local_settings_per_game()), Level::Error)
            .bind(|this| this.check_for_REFramework_update(), Level::Error)
            .bind(|this| this.ask_for_decision(), Level::Error)
            .bind(|this| {
                this.download_REFramework_update()
            }, Level::Error)
            .bind(|this| {
                this.unzip_updates()
            }, Level::Error)
            .save_config().unwrap();

        ()
    }
}

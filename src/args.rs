use clap::Parser;

use crate::{tomlConf::configStruct::ErrorLevel, ARGS};

pub fn convert_args_to_config() {
    let args: Vec<_> = std::env::args().collect();
    for argument in std::env::args() {
        println!("Argument {}", argument);
    }
    if args.len() > 2 && args[1] == "-run" {
        println!("About to run {} are you happy now?!:) ", args[2]);
    }
}
#[derive(Debug, clap::ValueEnum, Clone, Default)]
pub enum RunAfter {
    #[default]
    yes,
    no
}

impl RunAfter {
    pub fn to_bool(&self) -> bool {
        match self {
            RunAfter::yes => true,
            RunAfter::no => false,
        }
    }
}

/// Updater for reframework mod games
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct ArgsClap {
    /// Only performs update for one game mod and launch it later by default
    #[clap(short, long, value_parser, default_value = "none")]
    pub one: String,

    /// Debug level please use one of following: info, debug, warn, error, trace
    #[clap(short, long, value_enum, default_value = "none")]
    pub level: ErrorLevel,

    /// combined with one update
    #[clap(short, long, value_enum, default_value = "yes")]
    pub run: RunAfter,
}

pub unsafe fn parse_args() {
    let args = ArgsClap::parse();
    ARGS = Some(args);
}

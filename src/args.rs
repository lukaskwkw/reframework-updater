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
/// Updater for reframework mod games
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct ArgsClap {
    /// Shortname of game you want to launch
    #[clap(short, long, value_parser, default_value = "none")]
    pub run: String,

    /// Debug level please use one of following: info, debug, warn, error, trace
    #[clap(short, long, value_enum, default_value = "none")]
    pub level: ErrorLevel,
}

pub unsafe fn parse_args() {
    let args = ArgsClap::parse();
    ARGS = Some(args);
}

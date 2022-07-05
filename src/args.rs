use clap::Parser;
use env_logger::Env;

use crate::tomlConf::configStruct::ErrorLevel;

pub fn convert_args_to_config() {
    let args: Vec<_> = std::env::args().collect();
    for argument in std::env::args() {
        println!("Argument {}", argument);
    }
    if args.len() > 2 && args[1] == "-run" {
        print!("About to run {} are you happy now?!:) \n", args[2]);
    }
}
/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct ArgsClap {
    /// Shortname of game you want to launch
    #[clap(short, long, value_parser, default_value = "none")]
    run: String,

    /// Debug level please use one of following: info, debug, warn, error, trace
    #[clap(short, long, value_enum, default_value = "info")]
    level: ErrorLevel,
}

pub fn parse_args() {
    let args = ArgsClap::parse();

    let env = Env::default().filter_or(
        "NONENENENE",
        args.level.to_string()
    );
    env_logger::Builder::from_env(env).init();
}
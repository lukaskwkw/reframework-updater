use log::*;
use simplelog::*;
use std::fs::File;

pub fn init_logger(error_level: &str) {
    let logger = CombinedLogger::init(vec![
        // #[cfg(feature = "termcolor")]
        TermLogger::new(
            error_level.parse::<LevelFilter>().unwrap(),
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        // #[cfg(not(feature = "termcolor"))]
        SimpleLogger::new(LevelFilter::Warn, Config::default()),
        WriteLogger::new(
            error_level.parse::<LevelFilter>().unwrap(),
            Config::default(),
            File::create("refresher.log").unwrap(),
        ),
    ]);
    match logger {
        Ok(it) => it,
        Err(err) => {
            eprintln!("Logger error {:?}", err);
        }
    };
}

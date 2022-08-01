use log::*;
use simplelog::*;
use std::fs::File;

pub fn init_logger(error_level: &str) {
    let config = ConfigBuilder::new()
        .set_level_color(Level::Info, Some(Color::Rgb(102, 212, 0)))
        .build();

    let logger = CombinedLogger::init(vec![
        TermLogger::new(
            error_level.parse::<LevelFilter>().unwrap(),
            config,
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
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

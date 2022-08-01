use simple_log::LogConfigBuilder;

pub fn init_logger(error_level: &str) {
    let config = LogConfigBuilder::builder()
        .path("refresher.log")
        .size(1 * 100)
        .roll_count(10)
        .time_format("%Y-%m-%d %H:%M:%S") //E.g:%H:%M:%S.%f
        .level(error_level)
        .output_file()
        .output_console()
        .build();

    match simple_log::new(config) {
        Ok(it) => it,
        Err(err) => {
            eprintln!("Logger error {:?}", err);
        }
    };
}

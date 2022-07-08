use env_logger::Env;
use log::debug;

pub fn init_logger(error_level: &str) {
    let env = Env::default().filter_or(
        "REFR_LEVEL",
        error_level
    );

    match env_logger::Builder::from_env(env).try_init() {
        Ok(it) => it,
        Err(err) => {
            debug!("Logger already initialized {}", err);            
        }
    };
}

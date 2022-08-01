use std::{process, io};

use std::process::Command;

use std::env;

use crate::args::RunAfter;

pub fn restart_program(run_after: RunAfter, short_name: String) -> io::Result<()> {
    if cfg!(target_os = "windows") {
        let path = env::current_exe()?;
        Command::new(path)
            .args(["-r", &format!("{:?}", run_after), "--one", &short_name])
            .spawn()
            .expect("failed to execute process");
        process::exit(0);
    };
    Ok(())
}

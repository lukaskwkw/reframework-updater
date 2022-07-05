use mslnk::ShellLink;
use std::path::Path;

#[cfg(target_os = "windows")]
pub fn create_ms_lnk(lnk_name: impl AsRef<Path>, target: impl AsRef<Path>, arguments: Option<String>) {
    let mut sl = ShellLink::new(target).unwrap();
    sl.set_arguments(arguments);
    sl.create_lnk(lnk_name).unwrap();
}
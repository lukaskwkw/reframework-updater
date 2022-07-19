extern crate winres;

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_toolkit_path(r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64");
        res.set_icon("icon.ico");
        res.compile().unwrap();
    }
}

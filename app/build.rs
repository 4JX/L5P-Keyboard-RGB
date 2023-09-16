extern crate gio;
extern crate winres;

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon_with_id("./res/trayIcon.ico", "trayIcon");
        res.compile().unwrap();
    } else if cfg!(target_os = "linux") {
        gio::compile_resources("./res", "./res/tray-icon.xml", "compiled.gresource");
    }
}

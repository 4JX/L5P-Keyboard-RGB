fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();

        // Set the app icon on the executable
        res.set_icon("./res/trayIcon.ico");
        res.compile().unwrap();
    }
}

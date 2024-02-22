use eframe::egui::ViewportCommand;
use eframe::NativeOptions;


fn main() {
    env_logger::init();

    let app = App::new();

    eframe::run_native("Legion RGB", NativeOptions::default(), Box::new(|cc| Box::new(app))).unwrap();
}


pub struct App {}

impl App {
    pub fn new() -> Self {
        let app = Self {};

        app
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) {
            dbg!("foo");
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);

            ctx.send_viewport_cmd(ViewportCommand::Visible(false));
        }
    }
}

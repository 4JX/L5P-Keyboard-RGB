#![cfg_attr(not(test), windows_subsystem = "windows")]
#![cfg_attr(test, windows_subsystem = "console")]

mod cli;
#[cfg(target_os = "windows")]
mod console;
mod effects;
mod enums;
mod gui;
mod persist;
mod profile;
mod tray;
mod util;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use cli::{GuiCommand, OutputType};
use color_eyre::{eyre::eyre, Result};
use eframe::{egui::IconData, epaint::Vec2};
use gui::App;

const APP_ICON: &'static [u8; 14987] = include_bytes!("../res/trayIcon.ico");
const WINDOW_SIZE: Vec2 = Vec2::new(500., 400.);

fn main() {
    #[cfg(target_os = "windows")]
    {
        setup_panic().unwrap();

        // This just enables output if the program is already being ran from the CLI
        console::attach();
        let res = init();
        console::free();

        if res.is_err() {
            std::process::exit(2);
        }
    }

    #[cfg(target_os = "linux")]
    {
        color_eyre::install().unwrap();

        init().unwrap();
    }
}

#[cfg(target_os = "windows")]
fn setup_panic() -> Result<()> {
    // A somewhat unwrapped version of color_eyre::install() to add a "wait for enter" after printing the text
    use color_eyre::config::Theme;
    let builder = color_eyre::config::HookBuilder::default()
    // HACK: Forgo colors in windows outputs because I cannot figure out why allocated consoles don't display them
    .theme(Theme::new());

    let (panic_hook, eyre_hook) = builder.into_hooks();
    eyre_hook.install()?;

    std::panic::set_hook(Box::new(move |panic_info| {
        if !console::alloc() {
            // No point trying to print without a console...
            return;
        }

        eprintln!("{}", panic_hook.panic_report(panic_info));
        println!("Press Enter to continue...");
        let _ = std::io::stdin().read_line(&mut String::new());

        std::process::exit(1);
    }));

    Ok(())
}

fn init() -> Result<()> {
    let cli_output = cli::try_cli().map_err(|err| eyre!("{:?}", err))?;

    match cli_output {
        GuiCommand::Start { hide_window, output } => {
            start_ui(output, hide_window);

            Ok(())
        }
        GuiCommand::Exit => Ok(()),
    }
}

fn start_ui(output_type: OutputType, hide_window: bool) {
    let has_tray = Arc::new(AtomicBool::new(true));
    let visible = Arc::new(AtomicBool::new(!hide_window));

    let app_icon = load_icon_data(APP_ICON);
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size(WINDOW_SIZE)
            .with_min_inner_size(WINDOW_SIZE)
            .with_max_inner_size(WINDOW_SIZE)
            .with_icon(app_icon),
        ..eframe::NativeOptions::default()
    };

    let has_tray_c = has_tray.clone();

    // Since egui uses winit under the hood and doesn't use gtk on Linux, and we need gtk for
    // the tray icon to show up, we need to spawn a thread
    // where we initialize gtk and create the tray_icon
    {
        std::thread::spawn(move || {
            #[cfg(target_os = "linux")]
            gtk::init().unwrap();

            let tray_icon = tray::build_tray(true);
            has_tray_c.store(tray_icon.is_some(), Ordering::SeqCst);

            #[cfg(target_os = "linux")]
            gtk::main();
        });
    }

    let app = App::new(output_type, has_tray.clone(), visible.clone());

    eframe::run_native("Legion RGB", native_options, Box::new(move |cc| Ok(Box::new(app.init(cc))))).unwrap();
}

#[must_use]
fn load_icon_data(image_data: &[u8]) -> IconData {
    let image = image::load_from_memory(image_data).unwrap();
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.into_flat_samples().samples;

    IconData {
        rgba: pixels,
        width: image.width(),
        height: image.height(),
    }
}

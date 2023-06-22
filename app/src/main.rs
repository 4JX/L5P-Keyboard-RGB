mod cli;
mod effects;
mod enums;
mod gui;
mod persist;
mod profile;
mod util;

use cli::CliOutput;
use color_eyre::{eyre::eyre, Result};
use effects::EffectManager;
use eframe::{epaint::Vec2, IconData};
use gui::{App, GuiMessage};
use tray_item::{IconSource, TrayItem};
use util::is_unique_instance;

const WINDOW_SIZE: Vec2 = Vec2::new(500., 400.);

fn main() -> Result<()> {
    color_eyre::install()?;

    // Clear/Hide console if not running via one (Windows specific)
    #[cfg(target_os = "windows")]
    {
        #[link(name = "Kernel32")]
        extern "system" {
            fn GetConsoleProcessList(processList: *mut u32, count: u32) -> u32;
            fn FreeConsole() -> i32;
        }

        fn free_console() -> bool {
            unsafe { FreeConsole() == 0 }
        }

        fn is_console() -> bool {
            unsafe {
                let mut buffer = [0_u32; 1];
                let count = GetConsoleProcessList(buffer.as_mut_ptr(), 1);
                count != 1
            }
        }

        if !is_console() {
            free_console();
        }
    }

    let is_unique = is_unique_instance();

    let cli_output = cli::try_cli(is_unique).map_err(|err| eyre!("{:?}", err))?;

    if let Some(hide_window) = cli_output.start_gui_maybe_hidden {
        start_ui(cli_output, is_unique, hide_window);

        Ok(())
    } else {
        let mut effect_manager = EffectManager::new(effects::OperationMode::Cli).unwrap();

        match cli_output.output {
            cli::CliOutputType::Profile(profile) => {
                effect_manager.set_profile(profile);
                effect_manager.join_and_exit();
                Ok(())
            }
            cli::CliOutputType::Custom(effect) => {
                effect_manager.custom_effect(effect);
                effect_manager.join_and_exit();
                Ok(())
            }
            cli::CliOutputType::Exit => Ok(()),
            cli::CliOutputType::NoArgs => unreachable!("No arguments were provided but the app is in CLI mode"),
        }
    }
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

#[cfg(target_os = "linux")]
#[must_use]
pub fn load_tray_icon(image_data: &[u8]) -> IconSource {
    let image = image::load_from_memory(image_data).unwrap();
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.into_flat_samples().samples;

    IconSource::Data {
        data: pixels,
        width: image.width() as i32,
        height: image.height() as i32,
    }
}

fn start_ui(cli_output: CliOutput, is_unique: bool, hide_window: bool) {
    let app_icon = load_icon_data(include_bytes!("../res/trayIcon.ico"));
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(WINDOW_SIZE),
        min_window_size: Some(WINDOW_SIZE),
        max_window_size: Some(WINDOW_SIZE),
        icon_data: Some(app_icon),
        ..eframe::NativeOptions::default()
    };

    //Create the tray icon
    let (gui_sender, gui_receiver) = crossbeam_channel::unbounded::<GuiMessage>();

    #[cfg(target_os = "linux")]
    let tray_icon = load_tray_icon(include_bytes!("../res/trayIcon.ico"));

    #[cfg(target_os = "linux")]
    let tray_result = TrayItem::new("Keyboard RGB", tray_icon);

    #[cfg(target_os = "windows")]
    let tray_result = TrayItem::new("Keyboard RGB", IconSource::Resource("trayIcon"));

    let mut tray_item_err = false;

    if tray_result.is_ok() {
        let mut tray = tray_result.unwrap();

        let show_sender = gui_sender.clone();
        tray_item_err |= tray.add_menu_item("Show", move || show_sender.send(GuiMessage::ShowWindow).unwrap()).is_err();

        let quit_sender = gui_sender.clone();
        tray_item_err |= tray.add_menu_item("Quit", move || quit_sender.send(GuiMessage::Quit).unwrap()).is_err();
    } else {
        tray_item_err = true;
    }

    let gui_sender_clone = gui_sender.clone();

    let app = App::new(cli_output.output, hide_window, is_unique, !tray_item_err, gui_sender_clone, gui_receiver);

    eframe::run_native("Legion RGB", native_options, Box::new(|cc| Box::new(app.init(cc, gui_sender)))).unwrap();
}

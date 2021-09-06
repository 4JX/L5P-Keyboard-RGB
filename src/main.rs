#![windows_subsystem = "windows"]
use fltk::app;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tray_item::{IconSource, TrayItem};

mod gui;
mod keyboard_utils;

#[cfg(target_os = "windows")]
static SHOW_GUI: AtomicBool = AtomicBool::new(true);

fn main() {
	let app = app::App::default();
	let keyboard = match keyboard_utils::get_keyboard() {
		Ok(keyboard) => Arc::from(Mutex::from(keyboard)),
		Err(err) => panic!("{}", err),
	};

	#[cfg(target_os = "windows")]
	{
		//Create tray icon
		let mut tray = TrayItem::new("Backlight RGB", IconSource::Resource("trayIcon")).unwrap();

		tray.add_menu_item("Show", move || {
			SHOW_GUI.store(true, Ordering::Relaxed);
		})
		.unwrap();

		tray.add_menu_item("Quit", || {
			println!("Quit");
			std::process::exit(0);
		})
		.unwrap();

		//Tray loop
		loop {
			if SHOW_GUI.load(Ordering::Relaxed) {
				gui::start_ui(keyboard.clone());
				app.run().unwrap();
				SHOW_GUI.store(false, Ordering::Relaxed);
			} else {
				app::sleep(0.05);
			}
		}
	}

	#[cfg(not(target_os = "windows"))]
	{
		gui::start_ui(keyboard.clone());
		app.run().unwrap();
	}
}

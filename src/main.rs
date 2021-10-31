#![windows_subsystem = "windows"]
use fltk::app;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::Arc;
mod gui;
use gui::keyboard_manager;
mod keyboard_utils;
use gui::enums::Message;

fn main() {
	let app = app::App::default();

	let (tx, rx) = mpsc::channel::<Message>();
	let stop_signal = Arc::new(AtomicBool::new(false));
	let keyboard = match keyboard_utils::get_keyboard(stop_signal.clone()) {
		Ok(keyboard) => keyboard,
		Err(err) => panic!("{}", err),
	};
	let manager = keyboard_manager::KeyboardManager { keyboard, rx };

	//Windows tray logic
	#[cfg(target_os = "windows")]
	{
		use fltk::prelude::*;
		use tray_item::{IconSource, TrayItem};

		type HWND = *mut std::os::raw::c_void;

		static mut WINDOW: HWND = std::ptr::null_mut();

		let mut win = gui::builder::start_ui(manager, tx, stop_signal);

		unsafe {
			WINDOW = win.raw_handle();
		}
		win.set_callback(|_| {
			extern "C" {
				pub fn ShowWindow(hwnd: HWND, nCmdShow: i32) -> bool;
			}
			unsafe {
				ShowWindow(WINDOW, 0);
			}
		});
		//Create tray icon
		let mut tray = TrayItem::new("Keyboard RGB", IconSource::Resource("trayIcon")).unwrap();

		tray.add_menu_item("Show", move || {
			extern "C" {
				pub fn ShowWindow(hwnd: HWND, nCmdShow: i32) -> bool;
			}
			unsafe {
				ShowWindow(WINDOW, 9);
			}
		})
		.unwrap();

		tray.add_menu_item("Quit", || {
			println!("Quit");
			std::process::exit(0);
		})
		.unwrap();

		//Tray loop
		loop {
			if win.shown() {
				app.run().unwrap();
			} else {
				app::sleep(0.05);
			}
		}
	}

	#[cfg(not(target_os = "windows"))]
	{
		gui::builder::start_ui(manager, tx, stop_signal);
		app.run().unwrap();
	}
}

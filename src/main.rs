#![windows_subsystem = "windows"]
use fltk::{app, prelude::*};
use parking_lot::Mutex;
use std::sync::Arc;
use tray_item::{IconSource, TrayItem};

mod gui;
mod keyboard_utils;

type HWND = *mut std::os::raw::c_void;

static mut WINDOW: HWND = std::ptr::null_mut();

fn main() {
	let app = app::App::default();
	let keyboard = match keyboard_utils::get_keyboard() {
		Ok(keyboard) => Arc::from(Mutex::from(keyboard)),
		Err(err) => panic!("{}", err),
	};

	let mut win = gui::start_ui(keyboard.clone());

	//Windows tray logic
	#[cfg(target_os = "windows")]
	{
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
				pub fn ShowWindow(hwnd: crate::HWND, nCmdShow: i32) -> bool;
			}
			unsafe {
				ShowWindow(crate::WINDOW, 9);
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
	app.run().unwrap();
}

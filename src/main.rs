#![allow(dead_code)]

mod cli;
mod enums;
mod gui;
mod keyboard_manager;
mod keyboard_utils;

use color_eyre::Result;
use enums::{Effects, Message};
use fltk::app;
use keyboard_manager::KeyboardManager;
use std::env;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::Arc;

use crate::keyboard_manager::StopSignals;

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

	let (tx, rx) = mpsc::channel::<Message>();
	let keyboard_stop_signal = Arc::new(AtomicBool::new(false));
	let keyboard = match keyboard_utils::get_keyboard(keyboard_stop_signal.clone()) {
		Ok(keyboard) => keyboard,
		Err(err) => panic!("{}", err),
	};
	let mut manager = KeyboardManager {
		keyboard,
		rx,
		tx,
		stop_signals: StopSignals {
			manager_stop_signal: Arc::new(AtomicBool::new(false)),
			keyboard_stop_signal,
		},
		last_effect: Effects::Static,
	};

	cli::try_cli(&mut manager);

	let exec_name = env::current_exe().unwrap().file_name().unwrap().to_string_lossy().into_owned();
	println!("No subcommands found, starting in GUI mode. To view the possible subcommands type \"{} --help\".", exec_name);
	start_with_gui();

	Ok(())
}

fn start_with_gui() {
	let app = app::App::default();

	//Windows logic
	#[cfg(target_os = "windows")]
	{
		use fltk::prelude::*;
		use tray_item::{IconSource, TrayItem};

		type HWND = *mut std::os::raw::c_void;

		static mut WINDOW: HWND = std::ptr::null_mut();

		let mut win = gui::app::App::start_ui();

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

		loop {
			if win.shown() {
				app.run().unwrap();
			} else {
				app::sleep(0.05);
			}
		}
	}

	#[cfg(target_os = "linux")]
	{
		gui::app::App::start_ui();
		app.run().unwrap();
	}
}

#![windows_subsystem = "windows"]
mod gui;
mod keyboard_utils;

use clap::{crate_authors, crate_version, App, Arg, SubCommand};
use fltk::app;
use gui::enums::{Effects, Message};
use gui::keyboard_manager::KeyboardManager;
use std::convert::TryInto;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::Arc;
use std::{env, process};

fn main() {
	let (tx, rx) = mpsc::channel::<Message>();
	let stop_signal = Arc::new(AtomicBool::new(false));
	let keyboard = match keyboard_utils::get_keyboard(stop_signal.clone()) {
		Ok(keyboard) => keyboard,
		Err(err) => panic!("{}", err),
	};
	let mut manager = KeyboardManager { keyboard, rx };

	let matches = App::new("Legion Keyboard Control")
		.version(&crate_version!()[..])
		.author(&crate_authors!()[..])
		// .about("Placeholder")
		.arg(Arg::with_name("brightness").help("The brightness of the effect").takes_value(true).short("b").possible_values(&["1","2"]).default_value("1"))
		.arg(Arg::with_name("speed").help("The speed of the effect").takes_value(true).short("s").possible_values(&["1","2", "3", "4"]).default_value("1"))
		.subcommand(
			SubCommand::with_name("Static").about("Static effect").arg(
				Arg::with_name("colors")
					.help("List of 4 RGB triplets. Example: 255,0,0,255,255,0,0,0,255,255,128,0")
					.index(1)
					.required(true),
			),
		)
		.subcommand(
			SubCommand::with_name("Breath").about("Breath effect").arg(
				Arg::with_name("colors")
					.help("List of 4 RGB triplets. Example: 255,0,0,255,255,0,0,0,255,255,128,0")
					.index(1)
					.required(true),
			),
		)
		.subcommand(SubCommand::with_name("Smooth").about("Smooth effect"))
		.subcommand(SubCommand::with_name("LeftWave").about("Left Wave effect"))
		.subcommand(SubCommand::with_name("RightWave").about("Right Wave effect"))
		.subcommand(SubCommand::with_name("Lightning").about("Right Wave effect"))
		.subcommand(SubCommand::with_name("AmbientLight").about("Right Wave effect"))
		.subcommand(SubCommand::with_name("SmoothLeftWave").about("Right Wave effect"))
		.subcommand(SubCommand::with_name("SmoothRightWave").about("Right Wave effect"))
		.subcommand(
			SubCommand::with_name("LeftSwipe").about("Swipe effect").arg(
				Arg::with_name("colors")
					.help("List of 4 RGB triplets. Example: 255,0,0,255,255,0,0,0,255,255,128,0")
					.index(1)
					.required(true),
			),
		)
		.subcommand(
			SubCommand::with_name("RightSwipe").about("Swipe effect").arg(
				Arg::with_name("colors")
					.help("List of 4 RGB triplets. Example: 255,0,0,255,255,0,0,0,255,255,128,0")
					.index(1)
					.required(true),
			),
		)
		.get_matches();

	match matches.subcommand_name() {
		Some(input) => {
			let effect: Effects = Effects::from_str(input).unwrap();
			let speed = matches.value_of("speed").unwrap_or_default().parse::<u8>().unwrap_or(1);
			let brightness = matches.value_of("brightness").unwrap_or_default().parse::<u8>().unwrap_or(1);

			manager.keyboard.set_brightness(brightness);

			let matches = matches.subcommand_matches(input).unwrap();
			let color_array: [f32; 12] = match parse_bytes_arg(matches.value_of("colors").unwrap_or_default()).unwrap_or(vec![0.0; 12]).try_into() {
				Ok(color_array) => color_array,
				Err(_) => {
					println!("Invalid input, please check you used the correct format for the colors");
					process::exit(0);
				}
			};

			manager.set_effect_cli(effect, color_array, speed, &stop_signal);
		}
		None => {
			let exec_name = env::current_exe().unwrap().file_name().unwrap().to_string_lossy().into_owned();
			println!("No subcommands found, starting in GUI mode, to view the possible subcommands type \"{} --help\"", exec_name);
			start_with_gui(manager, tx, stop_signal);
		}
	}
	fn parse_bytes_arg(arg: &str) -> Result<Vec<f32>, <f32 as FromStr>::Err> {
		arg.split(',').map(|b| b.parse::<f32>()).collect()
	}
}

fn start_with_gui(manager: KeyboardManager, tx: mpsc::Sender<Message>, stop_signal: Arc<AtomicBool>) {
	let app = app::App::default();

	//Windows logic
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

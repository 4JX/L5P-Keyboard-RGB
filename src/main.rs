mod enums;
mod gui;
mod keyboard_manager;
mod keyboard_utils;

use clap::{crate_authors, crate_version, App, Arg, SubCommand};
use enums::{Effects, Message};
use fltk::app;
use keyboard_manager::KeyboardManager;
use std::convert::TryInto;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::Arc;
use std::{env, process};

fn main() {
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
	let stop_signal = Arc::new(AtomicBool::new(false));
	let keyboard = match keyboard_utils::get_keyboard(stop_signal.clone()) {
		Ok(keyboard) => keyboard,
		Err(err) => panic!("{}", err),
	};
	let mut manager = KeyboardManager {
		keyboard,
		rx,
		stop_signal: stop_signal.clone(),
		last_effect: Effects::Static,
	};

	let matches = App::new("Legion Keyboard Control")
		.version(crate_version!())
		.author(crate_authors!())
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
		.subcommand(SubCommand::with_name("Lightning").about("Lightning effect"))
		.subcommand(SubCommand::with_name("AmbientLight").about("AmbientLight effect"))
		.subcommand(SubCommand::with_name("SmoothLeftWave").about("SmoothLeftWave effect"))
		.subcommand(SubCommand::with_name("SmoothRightWave").about("SmoothRightWave effect"))
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
		.subcommand(SubCommand::with_name("Disco").about("Disco effect"))
		.get_matches();

	if let Some(input) = matches.subcommand_name() {
		fn parse_bytes_arg(arg: &str) -> Result<Vec<u8>, <u8 as FromStr>::Err> {
			arg.split(',').map(str::parse::<u8>).collect()
		}

		let effect: Effects = Effects::from_str(input).unwrap();
		let speed = matches.value_of("speed").unwrap_or_default().parse::<u8>().unwrap_or(1);
		let brightness = matches.value_of("brightness").unwrap_or_default().parse::<u8>().unwrap_or(1);

		let matches = matches.subcommand_matches(input).unwrap();

		let color_array: [u8; 12] = match effect {
			Effects::Static | Effects::Breath | Effects::LeftSwipe | Effects::RightSwipe => {
				let color_array = if let Some(value) = matches.value_of("colors") {
					parse_bytes_arg(value)
						.expect("Invalid input, please check you used the correct format for the colors")
						.try_into()
						.expect("Invalid input, please check you used the correct format for the colors")
				} else {
					println!("This effect requires specifying the colors to use.");
					process::exit(0);
				};
				color_array
			}
			_ => [0; 12],
		};

		manager.set_effect(effect, &color_array, speed, brightness);
	} else {
		let exec_name = env::current_exe().unwrap().file_name().unwrap().to_string_lossy().into_owned();
		println!("No subcommands found, starting in GUI mode, to view the possible subcommands type \"{} --help\"", exec_name);
		start_with_gui(manager, tx, stop_signal);
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

		let mut win = gui::app::App::start_ui(manager, tx, stop_signal);

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
		gui::app::App::start_ui(manager, tx, stop_signal);
		app.run().unwrap();
	}
}

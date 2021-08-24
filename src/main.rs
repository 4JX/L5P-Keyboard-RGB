#![windows_subsystem = "windows"]
mod gui;
mod keyboard_utils;

fn main() {
	let keyboard: keyboard_utils::Keyboard = match keyboard_utils::get_keyboard() {
		Ok(keyboard) => keyboard,
		Err(err) => panic!("{}", err),
	};
	gui::start_ui(keyboard);
}

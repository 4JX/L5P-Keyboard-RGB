#![windows_subsystem = "windows"]
mod gui;
mod keyboard_utils;

fn main() {
	let keyboard: crate::keyboard_utils::Keyboard = match crate::keyboard_utils::get_keyboard() {
		Ok(keyboard) => keyboard,
		Err(err) => panic!("{}", err),
	};
	gui::start_ui(keyboard);
}

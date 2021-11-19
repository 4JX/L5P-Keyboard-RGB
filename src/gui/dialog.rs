use fltk::{
	button::Button,
	enums::{Color, FrameType},
	frame,
	prelude::*,
	window::Window,
};

use super::{builder::center, enums::Colors};

pub fn message(width: i32, height: i32, message: &str) {
	let x = center().0 - width / 2;
	let y = center().1 - height / 2;
	let mut window = Window::new(x, y, width, height, "Message");
	window.set_color(Color::from_u32(Colors::DarkGray as u32));

	let _frame = frame::Frame::new(0, 0, width, height - 30 - 45, "")
		.with_label(message)
		.set_label_color(Color::from_u32(Colors::White as u32));

	let mut button = Button::new(width / 2 - 100 / 2, height - 30 - 45, 100, 45, "Close");
	button.set_color(Color::from_u32(Colors::Gray as u32));
	button.set_label_color(Color::White);
	button.set_frame(FrameType::BorderBox);
	window.end();
	window.show();

	button.set_callback({
		move |_but| {
			window.hide();
		}
	})
}

pub fn alert(width: i32, height: i32, message: &str) {
	let x = center().0 - width / 2;
	let y = center().1 - height / 2;
	let mut window = Window::new(x, y, width, height, "Message");
	window.set_color(Color::from_u32(Colors::DarkGray as u32));

	let _frame = frame::Frame::new(0, 0, width, height - 30 - 45, "")
		.with_label(message)
		.set_label_color(Color::from_u32(Colors::White as u32));

	let mut button = Button::new(width / 2 - 100 / 2, height - 30 - 45, 100, 45, "Close");
	button.set_color(Color::from_u32(Colors::Gray as u32));
	button.set_label_color(Color::White);
	button.set_frame(FrameType::BorderBox);
	window.end();
	window.show();

	button.set_callback({
		move |_but| {
			window.hide();
		}
	})
}

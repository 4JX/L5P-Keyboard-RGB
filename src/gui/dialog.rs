use super::{enums::Colors, utils::screen_center};
use fltk::{
	button::Button,
	enums::{Color, FrameType},
	prelude::*,
	text::WrapMode,
	window::Window,
};
use std::process;

#[allow(dead_code)]
pub fn message(width: i32, height: i32, message: &str) {
	let width_center = width / 2;
	let height_center = height / 2;
	let window_x = screen_center().0 - width_center;
	let window_y = screen_center().1 - height_center;
	let margin = 30;
	let button_width = 100;
	let button_height = 40;
	let display_width = width - margin * 2;
	let display_height = height - margin * 2 - button_height;

	let mut window = Window::new(window_x, window_y, width, height, "Message");
	window.set_color(Color::from_u32(Colors::DarkGray as u32));

	let mut buffer = fltk::text::TextBuffer::default();
	buffer.set_text(message);

	let mut display = fltk::text::TextDisplay::new(margin, margin, display_width, display_height, "");
	display.set_buffer(buffer);
	display.set_color(Color::from_u32(Colors::DarkGray as u32));
	display.set_frame(FrameType::FlatBox);
	display.set_text_color(Color::from_u32(Colors::White as u32));
	display.wrap_mode(WrapMode::AtBounds, 0);

	let mut button = Button::new(width_center - button_height / 2, height - margin - button_height + 10, button_width, button_height, "Close");
	button.set_color(Color::from_u32(Colors::Gray as u32));
	button.set_label_color(Color::White);
	button.set_frame(FrameType::BorderBox);

	window.end();
	window.show();
	window.make_resizable(false);

	button.set_callback({
		move |_but| {
			window.hide();
		}
	});
}

pub fn alert(width: i32, height: i32, message: &str, should_exit: bool) {
	let width_center = width / 2;
	let height_center = height / 2;
	let window_x = screen_center().0 - width_center;
	let window_y = screen_center().1 - height_center;
	let margin = 30;
	let button_width = 100;
	let button_height = 40;
	let display_width = width - margin * 2;
	let display_height = height - margin * 2 - button_height;

	let mut window = Window::new(window_x, window_y, width, height, "Warning");
	window.set_color(Color::from_u32(Colors::DarkGray as u32));

	let mut buffer = fltk::text::TextBuffer::default();
	buffer.set_text(message);

	let mut display = fltk::text::TextDisplay::new(margin, margin, display_width, display_height, "");
	display.set_buffer(buffer);
	display.set_color(Color::from_u32(Colors::DarkGray as u32));
	display.set_frame(FrameType::FlatBox);
	display.set_text_color(Color::from_u32(Colors::White as u32));
	display.wrap_mode(WrapMode::AtBounds, 0);

	let mut button = Button::new(width_center - button_height / 2, height - margin - button_height + 10, button_width, button_height, "Close");
	button.set_color(Color::from_u32(Colors::Gray as u32));
	button.set_label_color(Color::White);
	button.set_frame(FrameType::BorderBox);

	window.end();
	window.show();
	window.make_resizable(false);

	button.set_callback({
		move |_but| {
			if should_exit {
				process::exit(0);
			} else {
				window.hide();
			}
		}
	});
}

pub fn panic(width: i32, height: i32, message: &str) {
	let width_center = width / 2;
	let height_center = height / 2;
	let window_x = screen_center().0 - width_center;
	let window_y = screen_center().1 - height_center;
	let margin = 30;
	let button_width = 100;
	let button_height = 40;
	let note_height = 60;
	let display_width = width - margin * 2;
	let display_height = height - margin * 2 - button_height - note_height;

	let mut window = Window::new(window_x, window_y, width, height, "Something went wrong!");
	window.set_color(Color::from_u32(Colors::DarkRed as u32));

	let mut buffer = fltk::text::TextBuffer::default();
	buffer.set_text(message);

	let mut display = fltk::text::TextDisplay::new(margin, margin, display_width, display_height, "");
	display.set_buffer(buffer);
	display.set_color(Color::from_u32(Colors::DarkGray as u32));
	display.set_frame(FrameType::FlatBox);
	display.set_text_color(Color::from_u32(Colors::White as u32));

	let mut note_buffer = fltk::text::TextBuffer::default();
	note_buffer.set_text("The program encountered an error.\nPlease report the above error message, along with any relevant information to\nhttps://github.com/4JX/L5P-Keyboard-RGB/issues");
	let mut note_display = fltk::text::TextDisplay::new(margin, margin + display_height, display_width, note_height, "");
	note_display.set_buffer(note_buffer);
	note_display.set_color(Color::from_u32(Colors::Gray as u32));
	note_display.set_frame(FrameType::FlatBox);
	note_display.set_text_color(Color::from_u32(Colors::White as u32));

	let mut button = Button::new(width_center - button_height / 2, height - margin - button_height + 10, button_width, button_height, "Close");
	button.set_color(Color::from_u32(Colors::Gray as u32));
	button.set_label_color(Color::White);
	button.set_frame(FrameType::BorderBox);

	window.end();
	window.show();
	window.make_resizable(false);

	button.set_callback({
		move |_but| {
			process::exit(0);
		}
	});
}

pub fn about(width: i32, height: i32) {
	let width_center = width / 2;
	let height_center = height / 2;
	let window_x = screen_center().0 - width_center;
	let window_y = screen_center().1 - height_center;
	let margin = 30;
	let button_width = 100;
	let button_height = 40;
	let display_width = width - margin * 2;
	let display_height = height - margin * 2 - button_height + 20;

	let mut window = Window::new(window_x, window_y, width, height, "About");
	window.set_color(Color::from_u32(Colors::DarkGray as u32));

	let message = "A program made by 4JX.\n\nSomething's not working?: https://github.com/4JX/L5P-Keyboard-RGB\n\nDonate: https://liberapay.com/4JX/donate";
	let mut buffer = fltk::text::TextBuffer::default();
	buffer.set_text(message);

	let mut display = fltk::text::TextDisplay::new(margin + display_height + 20, margin, display_width - display_height, display_height, "");
	display.set_buffer(buffer);
	display.set_color(Color::from_u32(Colors::DarkGray as u32));
	display.set_frame(FrameType::FlatBox);
	display.set_text_color(Color::from_u32(Colors::White as u32));

	let mut button = Button::new(width_center - button_height / 2, height - margin - button_height + 15, button_width, button_height, "Close");
	button.set_color(Color::from_u32(Colors::Gray as u32));
	button.set_label_color(Color::White);
	button.set_frame(FrameType::BorderBox);

	let icon_str = include_str!("../../res/trayIcon.svg");
	let mut icon_svg = fltk::image::SvgImage::from_data(icon_str).unwrap();
	icon_svg.scale(100, 100, true, false);

	let image_shrink = 10;
	let mut image_frame = fltk::frame::Frame::new(margin + image_shrink, margin + image_shrink, display_height - image_shrink * 2, display_height - image_shrink * 2, "");
	image_frame.set_image_scaled(Some(icon_svg));

	window.end();
	window.show();
	window.make_resizable(false);

	button.set_callback({
		move |_but| {
			window.hide();
		}
	});
}

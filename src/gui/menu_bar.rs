use std::sync::{
	atomic::{AtomicBool, Ordering},
	mpsc, Arc,
};

use fltk::{
	dialog,
	enums::{Color, Font, FrameType, Shortcut},
	menu,
	prelude::*,
};

use crate::enums::Message;

use super::{builder::center, enums::Colors};

pub struct AppMenuBar {
	_menu: menu::SysMenuBar,
}

impl AppMenuBar {
	pub fn new(tx: &mpsc::Sender<Message>, stop_signal: Arc<AtomicBool>) -> Self {
		let mut menu = menu::SysMenuBar::default().with_size(900, 35);
		menu.set_color(Color::from_u32(Colors::DarkGray as u32));
		menu.set_selection_color(Color::from_u32(Colors::DarkerGray as u32));
		menu.set_frame(FrameType::FlatBox);
		menu.set_down_frame(FrameType::FlatBox);
		menu.set_text_font(Font::Helvetica);
		menu.set_text_color(Color::from_u32(Colors::White as u32));
		menu.add("&Profile/Save\t", Shortcut::None, menu::MenuFlag::Normal, {
			let tx = tx.clone();
			let stop_signal = stop_signal.clone();
			move |_some| {
				stop_signal.store(true, Ordering::SeqCst);
				tx.send(Message::SaveProfile).unwrap();
			}
		});
		menu.add("&Profile/Load\t", Shortcut::None, menu::MenuFlag::Normal, {
			let tx = tx.clone();
			move |_some| {
				stop_signal.store(true, Ordering::SeqCst);
				tx.send(Message::LoadProfile).unwrap();
			}
		});

		menu.add("&About\t", Shortcut::None, menu::MenuFlag::Normal, {
			move |_some| {
				dialog::message(
					center().0 - 100,
					center().1 - 100,
					"A program made by 4JX.\n\nFor updates, issues and feature requests, head over to:\nhttps://github.com/4JX/L5P-Keyboard-RGB/",
				);
			}
		});
		menu.add("Exit", Shortcut::None, menu::MenuFlag::Normal, {
			move |_some| {
				std::process::exit(0);
			}
		});

		if let Some(mut item) = menu.find_item("Exit") {
			item.set_label_color(Color::from_u32(Colors::Red as u32));
		}

		Self { _menu: menu }
	}
}

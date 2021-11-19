use std::sync::mpsc;

use fltk::{
	enums::{Color, Font, FrameType, Shortcut},
	menu,
	prelude::*,
};

use crate::enums::Message;

use super::enums::Colors;

pub struct AppMenuBar {
	_menu: menu::SysMenuBar,
}

impl AppMenuBar {
	pub fn new(tx: &mpsc::Sender<Message>) -> Self {
		let mut menu = menu::SysMenuBar::default().with_size(900, 35);
		menu.set_color(Color::from_u32(Colors::DarkGray as u32));
		menu.set_selection_color(Color::from_u32(Colors::DarkerGray as u32));
		menu.set_frame(FrameType::FlatBox);
		menu.set_down_frame(FrameType::FlatBox);
		menu.set_text_font(Font::Helvetica);
		menu.set_text_color(Color::from_u32(Colors::White as u32));
		menu.add("&Profile/Save\t", Shortcut::None, menu::MenuFlag::Normal, {
			let tx = tx.clone();
			move |_some| {
				tx.send(Message::Refresh).unwrap();
			}
		});
		menu.add("&Profile/Load\t", Shortcut::None, menu::MenuFlag::Normal, {
			let tx = tx.clone();
			move |_some| {
				tx.send(Message::Refresh).unwrap();
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

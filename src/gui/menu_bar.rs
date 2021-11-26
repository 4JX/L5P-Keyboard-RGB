use fltk::{
	enums::{Color, Font, FrameType, Shortcut},
	menu,
	prelude::*,
};

use super::{app::App, dialog, enums::Colors};

pub struct AppMenuBar {
	_menu: menu::SysMenuBar,
}

impl AppMenuBar {
	pub fn new(app: &App) -> Self {
		let mut menu = menu::SysMenuBar::default().with_size(900, 35);
		menu.set_color(Color::from_u32(Colors::DarkGray as u32));
		menu.set_selection_color(Color::from_u32(Colors::DarkerGray as u32));
		menu.set_frame(FrameType::FlatBox);
		menu.set_down_frame(FrameType::FlatBox);
		menu.set_text_font(Font::Helvetica);
		menu.set_text_color(Color::from_u32(Colors::White as u32));
		menu.add("&Profile/Save\t", Shortcut::None, menu::MenuFlag::Normal, {
			let mut app = app.clone();
			move |_some| {
				app.stop_signals.store_true();
				app.save_profile();
			}
		});
		menu.add("&Profile/Load\t", Shortcut::None, menu::MenuFlag::Normal, {
			let mut app = app.clone();
			move |_some| {
				app.stop_signals.store_true();
				app.load_profile(false);
			}
		});

		menu.add("&About", Shortcut::None, menu::MenuFlag::Normal, {
			move |_some| {
				dialog::about(800, 200);
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

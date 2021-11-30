use super::color_tiles::ColorTiles;
use super::options::OptionsTile;
use super::{color_tiles, effect_browser, options};
use crate::gui::menu_bar;
use crate::keyboard_manager::{self, StopSignals};
use crate::{
	enums::{Effects, Message},
	gui::dialog as appdialog,
};
use fltk::enums::FrameType;
use fltk::{app, enums::Font, prelude::*, window::Window};
use fltk::{
	browser::HoldBrowser,
	dialog,
	prelude::{BrowserExt, MenuExt},
	text,
};
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::time::Duration;
use std::{panic, thread};
use std::{path, str::FromStr, sync::mpsc::Sender};

const WIDTH: i32 = 900;
const HEIGHT: i32 = 480;

pub fn screen_center() -> (i32, i32) {
	((app::screen_size().0 / 2.0) as i32, (app::screen_size().1 / 2.0) as i32)
}

pub const EFFECTS_LIST: [&str; 15] = [
	"Static",
	"Breath",
	"Smooth",
	"LeftWave",
	"RightWave",
	"Lightning",
	"AmbientLight",
	"SmoothLeftWave",
	"SmoothRightWave",
	"LeftSwipe",
	"RightSwipe",
	"Disco",
	"Christmas",
	"Fade",
	"Temperature",
];

#[derive(Clone)]
pub struct App {
	pub color_tiles: ColorTiles,
	pub effect_browser: HoldBrowser,
	pub options_tile: OptionsTile,
	pub tx: Sender<Message>,
	pub stop_signals: StopSignals,
	pub buf: text::TextBuffer,
	pub center: (i32, i32),
}

impl App {
	pub fn load_profile(&mut self, is_default: bool) {}

	pub fn save_profile(&mut self) {}

	pub fn start_ui(manager: keyboard_manager::KeyboardManager) -> fltk::window::Window {
		panic::set_hook(Box::new(|info| {
			if let Some(s) = info.payload().downcast_ref::<&str>() {
				appdialog::panic(800, 400, s);
			} else {
				appdialog::panic(800, 400, &info.to_string());
			}
		}));

		let mut win = Window::new(screen_center().0 - WIDTH / 2, screen_center().1 - HEIGHT / 2, WIDTH, HEIGHT, "Legion Keyboard RGB Control");
		let tiles = color_tiles::ColorTiles::new(0, 30, &manager.tx, manager.stop_signals.clone());

		let app = Self {
			color_tiles: tiles,
			effect_browser: effect_browser::EffectBrowserTile::create(540, 30, &EFFECTS_LIST).effect_browser,
			options_tile: options::OptionsTile::create(540, 390, &manager.tx, &manager.stop_signals),
			tx: manager.tx.clone(),
			stop_signals: manager.stop_signals.clone(),
			buf: text::TextBuffer::default(),
			center: screen_center(),
		};

		menu_bar::AppMenuBar::new(&app);

		win.end();
		win.make_resizable(false);
		win.show();

		app::background(51, 51, 51);
		app::background2(119, 119, 119);
		app::foreground(0, 0, 0);
		app::set_visible_focus(false);
		app::set_font(Font::HelveticaBold);
		app::set_frame_border_radius_max(5);
		app::set_frame_type(FrameType::FlatBox);
		app::set_frame_type2(FrameType::DownBox, FrameType::RoundedBox);

		win
	}
}

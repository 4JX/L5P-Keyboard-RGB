use super::utils::screen_center;
use super::{color_tiles, effect_browser, options};
use crate::gui::dialog as appdialog;
use crate::gui::menu_bar;
use fltk::enums::FrameType;
use fltk::{app, enums::Font, prelude::*, window::Window};
use std::panic;

const WIDTH: i32 = 900;
const HEIGHT: i32 = 480;

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
pub struct App {}

impl App {
	pub fn load_profile(&mut self) {}

	pub fn save_profile(&mut self) {}

	pub fn start_ui() -> fltk::window::Window {
		panic::set_hook(Box::new(|info| {
			if let Some(s) = info.payload().downcast_ref::<&str>() {
				appdialog::panic(800, 400, s);
			} else {
				appdialog::panic(800, 400, &info.to_string());
			}
		}));

		let mut win = Window::new(screen_center().0 - WIDTH / 2, screen_center().1 - HEIGHT / 2, WIDTH, HEIGHT, "Legion Keyboard RGB Control");
		color_tiles::ColorTiles::new(0, 30);
		effect_browser::EffectBrowserTile::create(540, 30, &EFFECTS_LIST);
		options::OptionsTile::create(540, 390);

		menu_bar::AppMenuBar::new();

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

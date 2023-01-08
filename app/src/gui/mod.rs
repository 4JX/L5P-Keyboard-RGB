use eframe::{
	egui::{style::DebugOptions, CentralPanel, Context, Frame, Layout, ScrollArea, Style},
	emath::Align,
	epaint::{Color32, Rounding, Vec2},
	CreationContext,
};
use strum::IntoEnumIterator;

use tray_item::{IconSource, TrayItem};

use crate::{cli::CliOutputType, effects::EffectManager, enums::Effects, profile::Profile};

use self::{effect_options::EffectOptions, style::SpacingStyle};

mod effect_options;
mod style;

pub struct App {
	show_window: bool,
	window_open_rx: Option<crossbeam_channel::Receiver<GuiMessage>>,
	manager: EffectManager,
	profile: Profile,
	effect_options: EffectOptions,
	global_rgb: [u8; 3],
	spacing: SpacingStyle,
}

enum GuiMessage {
	ShowWindow,
}

impl App {
	pub fn new(manager: EffectManager, output: CliOutputType, hide_window: bool) -> Self {
		//Create the tray icon
		#[cfg(target_os = "linux")]
		let tray_icon = load_tray_icon(include_bytes!("../../res/trayIcon.ico"));

		#[cfg(target_os = "linux")]
		let mut tray = TrayItem::new("Keyboard RGB", tray_icon).unwrap();

		#[cfg(target_os = "windows")]
		let mut tray = TrayItem::new("Keyboard RGB", IconSource::Resource("trayIcon")).unwrap();

		let mut app = match output {
			CliOutputType::Profile(profile) => Self {
				show_window: !hide_window,
				window_open_rx: None,
				manager,
				profile,
				effect_options: EffectOptions::default(),
				global_rgb: [0; 3],
				spacing: SpacingStyle::default(),
			},
			CliOutputType::Custom(effect) => {
				// TODO: Handle custom effects
				let _ = effect;
				Self {
					show_window: !hide_window,
					window_open_rx: None,
					manager,
					profile: Profile::default(),
					effect_options: EffectOptions::default(),
					global_rgb: [0; 3],
					spacing: SpacingStyle::default(),
				}
			}
			CliOutputType::Exit => unreachable!("Exiting the app supersedes starting the GUI"),
		};

		let (window_sender, window_receiver) = crossbeam_channel::unbounded::<GuiMessage>();

		let mut tray_item_err = tray.add_menu_item("Show", move || window_sender.send(GuiMessage::ShowWindow).unwrap()).is_err();

		tray_item_err |= tray
			.add_menu_item("Quit", || {
				std::process::exit(0);
			})
			.is_err();

		if !tray_item_err {
			app.window_open_rx = Some(window_receiver);
		}

		app
	}

	pub fn init(self, cc: &CreationContext<'_>) -> App {
		self.configure_style(&cc.egui_ctx);
		self
	}
}

impl eframe::App for App {
	fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
		let mut update_lights = false;

		frame.set_visible(self.show_window);

		CentralPanel::default()
			.frame(Frame::none().inner_margin(self.spacing.large).fill(Color32::from_gray(26)))
			.show(ctx, |ui| {
				ui.style_mut().spacing.item_spacing = Vec2::splat(self.spacing.large);

				ui.with_layout(Layout::left_to_right(Align::Center).with_cross_justify(true), |ui| {
					ui.vertical(|ui| {
						let res = ui.scope(|ui| {
							ui.style_mut().spacing.item_spacing.y = self.spacing.medium;

							ui.set_enabled(self.profile.effect.takes_color_array());

							let response = ui.horizontal(|ui| {
								ui.style_mut().spacing.interact_size = Vec2::splat(60.0);

								for i in 0..4 {
									update_lights |= ui.color_edit_button_srgb(&mut self.profile.rgb_zones[i].rgb).changed();
								}
							});

							ui.style_mut().spacing.interact_size = Vec2::new(response.response.rect.width(), 30.0);

							if ui.color_edit_button_srgb(&mut self.global_rgb).changed() {
								for i in 0..4 {
									self.profile.rgb_zones[i].rgb = self.global_rgb;
								}

								update_lights = true;
							};

							response.response
						});

						ui.set_width(res.inner.rect.width());

						self.effect_options.show(ui, &mut self.profile, &mut update_lights, &self.spacing);

						ui.scope(|ui| {
							ui.style_mut().spacing.item_spacing = self.spacing.default;

							ui.horizontal(|ui| {
								ui.heading("Profiles");
								if ui.button("+").clicked() {}
								if ui.button("-").clicked() {}
							});

							Frame {
								rounding: Rounding::same(6.0),
								fill: Color32::from_gray(20),
								..Frame::default()
							}
							.show(ui, |ui| {
								ui.set_height(ui.available_height());

								ui.centered_and_justified(|ui| ui.label("No profiles added"));
							});
						});
					});

					Frame {
						rounding: Rounding::same(6.0),
						fill: Color32::from_gray(20),
						..Frame::default()
					}
					.show(ui, |ui| {
						ui.style_mut().spacing.item_spacing = self.spacing.default;

						ScrollArea::vertical().show(ui, |ui| {
							ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
								ui.add_space(ui.visuals().clip_rect_margin);

								for val in Effects::iter() {
									let text: &'static str = val.into();
									update_lights |= ui.selectable_value(&mut self.profile.effect, val, text).changed();
								}

								ui.add_space(ui.visuals().clip_rect_margin);
							});
						});
					});
				});
			});

		if update_lights {
			self.manager.set_profile(self.profile);
		}
	}

	fn on_close_event(&mut self) -> bool {
		if self.window_open_rx.is_some() {
			self.show_window = false;
			false
		} else {
			true
		}
	}
}

impl App {
	fn configure_style(&self, ctx: &Context) {
		let style = Style {
			// text_styles: text_utils::default_text_styles(),
			// visuals: THEME.visuals.clone(),
			debug: DebugOptions {
				debug_on_hover: false,
				show_expand_width: false,
				show_expand_height: false,
				show_resize: false,
				show_blocking_widget: false,
				show_interactive_widgets: false,
			},
			..Style::default()
		};

		// ctx.set_fonts(text_utils::get_font_def());
		ctx.set_style(style);
	}
}

#[must_use]
pub fn load_tray_icon(image_data: &[u8]) -> IconSource {
	let image = image::load_from_memory(image_data).unwrap();
	let image_buffer = image.to_rgba8();
	let pixels = image_buffer.as_raw().clone();

	IconSource::Data {
		data: pixels,
		width: image.width() as i32,
		height: image.height() as i32,
	}
}

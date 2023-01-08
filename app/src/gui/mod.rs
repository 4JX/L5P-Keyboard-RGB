use std::process;

use eframe::{
	egui::{style::DebugOptions, CentralPanel, Context, Frame, Layout, ScrollArea, Style},
	emath::Align,
	epaint::{Color32, Rounding, Vec2},
	CreationContext,
};
use egui_modal::Modal;
use strum::IntoEnumIterator;

use tray_item::{IconSource, TrayItem};

use crate::{
	cli::CliOutputType,
	effects::{self, EffectManager},
	enums::Effects,
	profile::Profile,
};

use self::{effect_options::EffectOptions, profile_list::ProfileList, style::SpacingStyle};

mod effect_options;
mod profile_list;
mod style;

pub struct App {
	unique_instance: bool,
	show_window: bool,
	window_open_rx: Option<crossbeam_channel::Receiver<GuiMessage>>,

	manager: Option<EffectManager>,

	profile_list: ProfileList,
	profile: Profile,
	effect_options: EffectOptions,
	global_rgb: [u8; 3],

	spacing: SpacingStyle,
}

enum GuiMessage {
	ShowWindow,
}

impl App {
	pub fn new(output: CliOutputType, hide_window: bool, unique_instance: bool) -> Self {
		// TODO: Handle errors visually
		let manager = EffectManager::new(effects::OperationMode::Gui).ok();

		let mut app = match output {
			CliOutputType::Profile(profile) => Self {
				unique_instance,
				show_window: !hide_window,
				window_open_rx: None,
				manager,
				profile_list: ProfileList::default(),
				profile,
				effect_options: EffectOptions::default(),
				global_rgb: [0; 3],
				spacing: SpacingStyle::default(),
			},
			CliOutputType::Custom(effect) => {
				// TODO: Handle custom effects
				let _ = effect;
				Self {
					unique_instance,
					show_window: !hide_window,
					window_open_rx: None,
					manager,
					profile_list: ProfileList::default(),
					profile: Profile::default(),
					effect_options: EffectOptions::default(),
					global_rgb: [0; 3],
					spacing: SpacingStyle::default(),
				}
			}
			CliOutputType::Exit => unreachable!("Exiting the app supersedes starting the GUI"),
		};

		//Create the tray icon
		#[cfg(target_os = "linux")]
		let tray_icon = load_tray_icon(include_bytes!("../../res/trayIcon.ico"));

		#[cfg(target_os = "linux")]
		let mut tray = TrayItem::new("Keyboard RGB", tray_icon).unwrap();

		#[cfg(target_os = "windows")]
		let mut tray = TrayItem::new("Keyboard RGB", IconSource::Resource("trayIcon")).unwrap();

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
		if !self.unique_instance {
			dbg!("not unique");
			if self.manager.is_none() {
				let modal = Modal::new(ctx, "unique_instance_error_modal");

				modal.show(|ui| {
					modal.title(ui, "Warning");
					modal.frame(ui, |ui| {
						modal.body(ui, "Another instance is already running, please close it and try again.");
					});

					modal.buttons(ui, |ui| {
						if modal.caution_button(ui, "Exit").clicked() {
							process::exit(0)
						}
					});
				});

				modal.open()
			}
		}

		if self.manager.is_none() {
			let modal = Modal::new(ctx, "manager_error_modal");

			modal.show(|ui| {
				modal.title(ui, "Warning");
				modal.frame(ui, |ui| {
					modal.body(ui, "Failed to find a valid keyboard.");
					modal.body(ui, "Ensure that you have a supported model and that the application has access to it.");
					modal.body(ui, "On Linux, see https://github.com/4JX/L5P-Keyboard-RGB#usage");
					modal.body(ui, "In certain cases, this may be due to a hardware error.");
				});

				modal.buttons(ui, |ui| {
					if modal.caution_button(ui, "Exit").clicked() {
						process::exit(0)
					}
				});
			});

			modal.open()
		}

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

						self.profile_list.show(ctx, ui, &mut self.profile, &self.spacing);
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
			if let Some(manager) = self.manager.as_mut() {
				manager.set_profile(self.profile.clone());
			}
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

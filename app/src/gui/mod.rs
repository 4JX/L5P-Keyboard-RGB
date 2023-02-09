use std::{
    mem,
    path::{Path, PathBuf},
    process,
};

use crossbeam_channel::{Receiver, Sender};
use eframe::{
    egui::{style::DebugOptions, CentralPanel, Context, Frame, Layout, ScrollArea, Style, TopBottomPanel},
    emath::Align,
    epaint::{Color32, Rounding, Vec2},
    CreationContext,
};

use strum::IntoEnumIterator;

use crate::{
    cli::CliOutputType,
    effects::{self, custom_effect::CustomEffect, EffectManager},
    enums::Effects,
    persist::{Settings, Updates},
    profile::Profile,
    util::StorageTrait,
};

use self::{effect_options::EffectOptions, menu_bar::MenuBarState, profile_list::ProfileList, style::Theme};

mod effect_options;
mod menu_bar;
mod modals;
mod profile_list;
mod style;

pub struct App {
    unique_instance: bool,
    show_window: bool,
    window_open_rx: Option<crossbeam_channel::Receiver<GuiMessage>>,
    update_data: Updates,
    show_update_modal: bool,

    manager: Option<EffectManager>,
    profile: Profile,
    custom_effect: CustomEffectState,

    menu_bar: MenuBarState,
    profile_list: ProfileList,
    effect_options: EffectOptions,
    global_rgb: [u8; 3],
    theme: Theme,
}

pub enum GuiMessage {
    ShowWindow,
    Quit,
}

#[derive(Default)]
pub enum CustomEffectState {
    #[default]
    None,
    Queued(CustomEffect),
    Playing,
}

impl CustomEffectState {
    fn is_none(&self) -> bool {
        match self {
            Self::None => true,
            Self::Queued(_) | Self::Playing => false,
        }
    }

    fn is_queued(&self) -> bool {
        match self {
            Self::None | Self::Playing => false,
            Self::Queued(_) => true,
        }
    }

    fn is_playing(&self) -> bool {
        match self {
            Self::None | Self::Queued(_) => false,
            Self::Playing => true,
        }
    }
}

impl App {
    pub fn new(output: CliOutputType, hide_window: bool, unique_instance: bool, tray_active: bool, tx: Sender<GuiMessage>, rx: Receiver<GuiMessage>) -> Self {
        let manager = EffectManager::new(effects::OperationMode::Gui).ok();

        let settings: Settings = Settings::load_with_check(Path::new("./settings.json"));

        let mut app = match output {
            CliOutputType::Profile(profile) => Self {
                unique_instance,
                show_window: !hide_window,
                window_open_rx: None,
                update_data: settings.updates.clone(),
                show_update_modal: true,

                manager,
                profile,
                custom_effect: CustomEffectState::default(),

                menu_bar: MenuBarState::new(tx),
                profile_list: ProfileList::new(settings.profiles),
                effect_options: EffectOptions::default(),
                global_rgb: [0; 3],
                theme: Theme::default(),
            },
            CliOutputType::Custom(effect) => Self {
                unique_instance,
                show_window: !hide_window,
                window_open_rx: None,
                update_data: settings.updates.clone(),
                show_update_modal: true,

                manager,
                profile: Profile::default(),
                custom_effect: CustomEffectState::Queued(effect),

                menu_bar: MenuBarState::new(tx),
                profile_list: ProfileList::new(settings.profiles),
                effect_options: EffectOptions::default(),
                global_rgb: [0; 3],
                theme: Theme::default(),
            },
            CliOutputType::Exit => unreachable!("Exiting the app supersedes starting the GUI"),
        };

        if tray_active {
            app.window_open_rx = Some(rx);
        }

        app
    }

    pub fn init(self, cc: &CreationContext<'_>) -> Self {
        self.configure_style(&cc.egui_ctx);
        self
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        if let Some(rx) = &self.window_open_rx {
            if let Ok(message) = rx.try_recv() {
                match message {
                    GuiMessage::ShowWindow => self.show_window = true,
                    GuiMessage::Quit => self.exit_app(),
                }
            }
        }

        if !self.update_data.skip_version {
            if let Some(update_name) = self.update_data.version_name.as_ref() {
                modals::update_available(ctx, update_name, &mut self.update_data.skip_version, &mut self.show_update_modal);
            }
        }

        if !self.unique_instance && modals::unique_instance(ctx) {
            self.exit_app();
        };

        if self.manager.is_none() && modals::manager_error(ctx) {
            self.exit_app();
        };

        let mut changed = false;

        frame.set_visible(self.show_window);

        TopBottomPanel::top("top-panel").show(ctx, |ui| {
            self.menu_bar.show(ctx, ui, &mut self.profile, &mut self.custom_effect, &mut changed);
        });

        CentralPanel::default()
            .frame(Frame::none().inner_margin(self.theme.spacing.large).fill(Color32::from_gray(26)))
            .show(ctx, |ui| {
                ui.style_mut().spacing.item_spacing = Vec2::splat(self.theme.spacing.large);

                ui.with_layout(Layout::left_to_right(Align::Center).with_cross_justify(true), |ui| {
                    ui.vertical(|ui| {
                        let res = ui.scope(|ui| {
                            ui.set_enabled(self.profile.effect.takes_color_array() && self.custom_effect.is_none());

                            ui.style_mut().spacing.item_spacing.y = self.theme.spacing.medium;

                            let response = ui.horizontal(|ui| {
                                ui.style_mut().spacing.interact_size = Vec2::splat(60.0);

                                for i in 0..4 {
                                    changed |= ui.color_edit_button_srgb(&mut self.profile.rgb_zones[i].rgb).changed();
                                }
                            });

                            ui.style_mut().spacing.interact_size = Vec2::new(response.response.rect.width(), 30.0);

                            if ui.color_edit_button_srgb(&mut self.global_rgb).changed() {
                                for i in 0..4 {
                                    self.profile.rgb_zones[i].rgb = self.global_rgb;
                                }

                                changed = true;
                            };

                            response.response
                        });

                        ui.set_width(res.inner.rect.width());

                        ui.scope(|ui| {
                            ui.set_enabled(self.custom_effect.is_none());
                            self.effect_options.show(ui, &mut self.profile, &mut changed, &self.theme.spacing);
                        });

                        self.profile_list.show(ctx, ui, &mut self.profile, &self.theme.spacing, &mut changed, &mut self.custom_effect);
                    });

                    ui.vertical_centered_justified(|ui| {
                        if self.custom_effect.is_playing() && ui.button("Stop custom effect").clicked() {
                            self.custom_effect = CustomEffectState::None;
                            changed = true;
                        };

                        Frame {
                            rounding: Rounding::same(6.0),
                            fill: Color32::from_gray(20),
                            ..Frame::default()
                        }
                        .show(ui, |ui| {
                            ui.style_mut().spacing.item_spacing = self.theme.spacing.default;

                            ScrollArea::vertical().show(ui, |ui| {
                                ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                                    for val in Effects::iter() {
                                        let text: &'static str = val.into();
                                        if ui.selectable_value(&mut self.profile.effect, val, text).changed() {
                                            changed = true;
                                            self.custom_effect = CustomEffectState::None;
                                        };
                                    }
                                });
                            });
                        });
                    });
                });
            });

        if changed {
            if let Some(manager) = self.manager.as_mut() {
                if self.custom_effect.is_none() {
                    manager.set_profile(self.profile.clone());
                } else if self.custom_effect.is_queued() {
                    let state = mem::replace(&mut self.custom_effect, CustomEffectState::Playing);
                    if let CustomEffectState::Queued(effect) = state {
                        manager.custom_effect(effect);
                    }
                }
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

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let path = PathBuf::from("./settings.json");

        let mut settings = Settings::load_or_default(&path);

        settings.profiles = std::mem::take(&mut self.profile_list.profiles);

        settings.updates = std::mem::take(&mut self.update_data);

        settings.save(path).unwrap();
    }
}

impl App {
    fn configure_style(&self, ctx: &Context) {
        let style = Style {
            // text_styles: text_utils::default_text_styles(),
            visuals: self.theme.visuals.clone(),
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

    fn exit_app(&mut self) {
        use eframe::App;

        self.on_exit(None);

        process::exit(0);
    }
}

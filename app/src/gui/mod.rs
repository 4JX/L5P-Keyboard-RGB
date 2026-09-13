use std::{process, thread, time::Duration};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use device_query::{DeviceQuery, Keycode};
#[cfg(debug_assertions)]
use eframe::egui::style::DebugOptions;
use eframe::{
    egui::{CentralPanel, Context, CornerRadius, Frame, Layout, ScrollArea, Style, TopBottomPanel, ViewportCommand},
    emath::Align,
    epaint::{Color32, Vec2},
    CreationContext,
};

use egui_notify::Toasts;
use strum::IntoEnumIterator;
use tray_icon::menu::MenuEvent;

use crate::{
    cli::OutputType,
    enums::Effects,
    manager::{self, custom_effect::CustomEffect, profile::Profile, show_effect_ui, EffectManager, ManagerCreationError},
    persist::Settings,
    tray::{QUIT_ID, SHOW_ID},
    DENY_HIDING,
};

use self::{menu_bar::MenuBarState, saved_items::SavedItems, style::Theme};

mod menu_bar;
mod modals;
mod saved_items;
pub mod style;

pub struct App {
    instance_not_unique: bool,
    gui_tx: crossbeam_channel::Sender<GuiMessage>,
    gui_rx: crossbeam_channel::Receiver<GuiMessage>,

    has_tray: Arc<AtomicBool>,
    visible: Arc<AtomicBool>,
    was_minimized: bool,

    manager: Option<EffectManager>,
    state_changed: bool,
    loaded_effect: LoadedEffect,
    current_profile: Profile,

    menu_bar: MenuBarState,
    saved_items: SavedItems,
    global_rgb: [u8; 3],
    theme: Theme,
    toasts: Toasts,
    pub app_settings: crate::settings::Settings,
}

pub enum GuiMessage {
    CycleProfiles,
    Quit,
}

pub struct LoadedEffect {
    state: State,
    effect: CustomEffect,
}

impl LoadedEffect {
    pub fn default() -> Self {
        Self::none()
    }

    pub fn none() -> Self {
        Self {
            state: State::None,
            effect: CustomEffect::default(),
        }
    }

    pub fn queued(effect: CustomEffect) -> Self {
        Self { state: State::Queued, effect }
    }

    pub fn is_none(&self) -> bool {
        matches!(self.state, State::None)
    }

    pub fn is_queued(&self) -> bool {
        matches!(self.state, State::Queued)
    }

    pub fn is_playing(&self) -> bool {
        matches!(self.state, State::Playing)
    }
}

#[derive(Default)]
pub enum State {
    #[default]
    None,
    Queued,
    Playing,
}

impl App {
    pub fn new(output: OutputType, has_tray: Arc<AtomicBool>, visible: Arc<AtomicBool>) -> Self {
        let (gui_tx, gui_rx) = crossbeam_channel::unbounded::<GuiMessage>();

        let manager_result = EffectManager::new(manager::OperationMode::Gui);

        let instance_not_unique = if let Err(err) = &manager_result {
            &ManagerCreationError::InstanceAlreadyRunning == err.current_context()
        } else {
            false
        };

        if instance_not_unique {
            std::process::exit(0);
        }

        let manager = manager_result.ok();

        let app_settings = crate::settings::Settings::load();
        
        let current_profile = if let Some(ref p) = app_settings.saved_profile {
            p.clone()
        } else if let Some(ref p) = app_settings.last_profile {
            p.clone()
        } else {
            let legacy_settings = Settings::load();
            legacy_settings.current_profile
        };

        let profiles = if !app_settings.profiles.is_empty() {
            app_settings.profiles.clone()
        } else {
            let legacy_settings = Settings::load();
            legacy_settings.profiles
        };

        let effects = if !app_settings.effects.is_empty() {
            app_settings.effects.clone()
        } else {
            let legacy_settings = Settings::load();
            legacy_settings.effects
        };

        let gui_tx_c = gui_tx.clone();
        // Default app state
        let mut app = Self {
            instance_not_unique,
            gui_tx,
            gui_rx,

            has_tray,
            visible,
            was_minimized: false,

            manager,
            // Default to true for an instant update on launch
            state_changed: true,
            loaded_effect: LoadedEffect::default(),
            current_profile,

            menu_bar: MenuBarState::new(gui_tx_c),
            saved_items: SavedItems::new(profiles, effects),
            global_rgb: [0; 3],
            theme: Theme::default(),
            toasts: Toasts::default(),
            app_settings,
        };

        // Update the state according to the option chosen by the user
        match output {
            OutputType::Profile(profile) => app.current_profile = profile,
            OutputType::Custom(effect) => app.loaded_effect = LoadedEffect::queued(effect),
            OutputType::NoArgs => {}
            OutputType::Exit => unreachable!("Exiting the app supersedes starting the GUI"),
        }

        app
    }

    pub fn init(self, cc: &CreationContext<'_>) -> Self {
        self.configure_style(&cc.egui_ctx);

        let egui_ctx = cc.egui_ctx.clone();
        let _gui_tx = self.gui_tx.clone();
        let _has_tray = self.has_tray.clone();
        let visible_c = self.visible.clone();

        let wake_socket = std::net::UdpSocket::bind("127.0.0.1:48294").ok();
        if let Some(sock) = &wake_socket {
            sock.set_nonblocking(true).unwrap();
        }

        std::thread::spawn(move || loop {
            if let Ok(event) = MenuEvent::receiver().try_recv() {
                if event.id == SHOW_ID {
                    visible_c.store(true, Ordering::SeqCst);
                    egui_ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                    egui_ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
                    egui_ctx.send_viewport_cmd(ViewportCommand::Focus);
                    egui_ctx.request_repaint();
                } else if event.id == QUIT_ID {
                    std::process::exit(0);
                }
            }

            #[cfg(not(target_os = "linux"))]
            if let Ok(event) = tray_icon::TrayIconEvent::receiver().try_recv() {
                if let tray_icon::TrayIconEvent::Click { button: tray_icon::MouseButton::Left, .. } = event {
                    visible_c.store(true, Ordering::SeqCst);
                    egui_ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                    egui_ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
                    egui_ctx.send_viewport_cmd(ViewportCommand::Focus);
                    egui_ctx.request_repaint();
                }
            }

            if let Some(sock) = &wake_socket {
                let mut buf = [0; 10];
                if let Ok((amt, _)) = sock.recv_from(&mut buf) {
                    if &buf[..amt] == b"WAKE" {
                        visible_c.store(true, Ordering::SeqCst);
                        egui_ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                        egui_ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
                        egui_ctx.send_viewport_cmd(ViewportCommand::Focus);
                        egui_ctx.request_repaint();
                    }
                }
            }

            std::thread::sleep(Duration::from_millis(50));
        });

        let ctx = cc.egui_ctx.clone();
        let gui_tx_c = self.gui_tx.clone();
        if self.manager.is_some() {
            thread::spawn(move || {
                let state = device_query::DeviceState::new();
                let mut lock_switching = false;

                loop {
                    let keys = state.get_keys();

                    if keys.contains(&Keycode::LMeta) && keys.contains(&Keycode::RAlt) {
                        if !lock_switching {
                            let _ = gui_tx_c.send(GuiMessage::CycleProfiles);
                            ctx.request_repaint();
                            lock_switching = true;
                        }
                    } else {
                        lock_switching = false;
                    }

                    thread::sleep(Duration::from_millis(50));
                }
            });
        }

        self.configure_style(&cc.egui_ctx);

        self
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(message) = self.gui_rx.try_recv() {
            match message {
                GuiMessage::CycleProfiles => self.cycle_profiles(),
                GuiMessage::Quit => self.exit_app(),
            }
        }

        // Show active toast messages
        self.toasts.show(ctx);

        if *DENY_HIDING && !self.visible.load(Ordering::SeqCst) {
            self.visible.store(true, Ordering::SeqCst);
            self.toasts
                .warning("Window hiding is currently not supported.\nSee https://github.com/4JX/L5P-Keyboard-RGB/issues/181")
                .duration(None);
        }

        if self.instance_not_unique && modals::unique_instance(ctx) {
            self.exit_app();
        }

        if !self.instance_not_unique && self.manager.is_none() && modals::manager_error(ctx) {
            self.exit_app();
        }

        TopBottomPanel::top("top-panel").show(ctx, |ui| {
            self.menu_bar.show(
                ctx,
                ui,
                &mut self.current_profile,
                &mut self.loaded_effect,
                &mut self.state_changed,
                &mut self.toasts,
                &mut self.app_settings,
                &mut self.saved_items,
            );
        });

        CentralPanel::default()
            .frame(Frame::new().inner_margin(self.theme.spacing.large).fill(Color32::from_rgba_unmultiplied(26, 26, 26, 200)))
            .show(ctx, |ui| {
                ui.style_mut().spacing.item_spacing = Vec2::splat(self.theme.spacing.large);
                self.show_ui_elements(ctx, ui);
            });

        if self.state_changed {
            self.update_state();
        }

        self.handle_close_request(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.app_settings.last_profile = Some(self.current_profile.clone());
        self.app_settings.profiles = self.saved_items.profiles.clone();
        self.app_settings.effects = self.saved_items.custom_effects.clone();
        self.app_settings.save();

        let SavedItems { profiles, custom_effects, .. } = self.saved_items.clone();

        let mut settings = Settings::new(profiles, custom_effects, self.current_profile.clone());

        settings.save();

        self.visible.store(false, Ordering::SeqCst);

        if let Some(manager) = self.manager.take() {
            manager.shutdown();
        }
    }
}

impl App {
    fn configure_style(&self, ctx: &Context) {
        let style = Style {
            // text_styles: text_utils::default_text_styles(),
            visuals: self.theme.visuals.clone(),
            #[cfg(debug_assertions)]
            debug: DebugOptions {
                debug_on_hover: false,
                debug_on_hover_with_all_modifiers: false,
                hover_shows_next: false,
                show_expand_width: false,
                show_expand_height: false,
                show_resize: false,
                show_interactive_widgets: false,
                show_widget_hits: false,
                show_unaligned: false,
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

    fn cycle_profiles(&mut self) {
        let len = self.saved_items.profiles.len();

        let current_profile_name = &self.current_profile.name;

        if let Some((i, _)) = self.saved_items.profiles.iter().enumerate().find(|(_, profile)| &profile.name == current_profile_name) {
            if i == len - 1 && len > 0 {
                self.current_profile = self.saved_items.profiles[0].clone();
            } else {
                self.current_profile = self.saved_items.profiles[i + 1].clone();
            }

            self.state_changed = true;
        }
    }

    fn show_ui_elements(&mut self, ctx: &Context, ui: &mut eframe::egui::Ui) {
        ui.with_layout(Layout::left_to_right(Align::Center).with_cross_justify(true), |ui| {
            ui.vertical(|ui| {
                let can_tweak_colors = self.current_profile.effect.takes_color_array() && self.loaded_effect.is_none();

                let res = ui.add_enabled_ui(can_tweak_colors, |ui| {
                    ui.style_mut().spacing.item_spacing = Vec2::splat(self.theme.spacing.medium);
                    let response = ui.horizontal(|ui| {
                        ui.style_mut().spacing.interact_size = Vec2::new(70.0, 50.0);

                        for i in 0..4 {
                            self.state_changed |= ui.color_edit_button_srgb(&mut self.current_profile.rgb_zones[i].rgb).changed();
                        }
                    });

                    ui.style_mut().spacing.interact_size = Vec2::new(response.response.rect.width(), 30.0);
                    if ui.color_edit_button_srgb(&mut self.global_rgb).changed() {
                        for i in 0..4 {
                            self.current_profile.rgb_zones[i].rgb = self.global_rgb;
                        }
                        self.state_changed = true;
                    }

                    ui.horizontal_wrapped(|ui| {
                        ui.label(eframe::egui::RichText::new("🎨 Palettes:").small().color(Color32::from_gray(180)));
                        for preset in crate::manager::profile::PRESET_PALETTES {
                            let btn = ui.add(eframe::egui::Button::new(eframe::egui::RichText::new(preset.name).small()))
                                .on_hover_text(format!("Apply {} palette", preset.name));
                            if btn.clicked() {
                                self.current_profile.rgb_zones = crate::manager::profile::arr_to_zones(preset.colors);
                                self.state_changed = true;
                                self.toasts.success(format!("Applied {} palette!", preset.name)).duration(Some(Duration::from_millis(2000))).closable(true);
                            }
                        }
                    });

                    response.response
                });

                ui.set_width(res.inner.rect.width());

                self.show_effect_ui(ui);

                self.saved_items
                    .show(ctx, ui, &mut self.current_profile, &mut self.loaded_effect, &self.theme.spacing, &mut self.state_changed);
            });

            ui.vertical_centered_justified(|ui| {
                if self.loaded_effect.is_playing() && ui.button("Stop custom effect").clicked() {
                    self.loaded_effect.state = State::None;
                    self.state_changed = true;
                }

                Frame {
                    corner_radius: CornerRadius::same(6),
                    fill: Color32::from_gray(20),
                    ..Frame::default()
                }
                .show(ui, |ui| {
                    ui.style_mut().spacing.item_spacing = self.theme.spacing.default;
                    ScrollArea::vertical().show(ui, |ui| {
                        ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                            for val in Effects::iter() {
                                let text: &'static str = val.into();
                                let is_selected = self.current_profile.effect == val;
                                if ui.selectable_label(is_selected, text).clicked() {
                                    if !is_selected {
                                        self.current_profile.effect = val.with_sensible_defaults();
                                        self.state_changed = true;
                                        self.loaded_effect.state = State::None;
                                    }
                                }
                            }
                        });
                    });
                });
            });
        });
    }

    fn show_effect_ui(&mut self, ui: &mut eframe::egui::Ui) {
        ui.add_enabled_ui(self.loaded_effect.is_none(), |ui| {
            show_effect_ui(ui, &mut self.current_profile, &mut self.state_changed, &self.theme);
        });
    }

    fn update_state(&mut self) {
        if let Some(manager) = self.manager.as_mut() {
            if self.loaded_effect.is_none() {
                manager.set_profile(self.current_profile.clone());
            } else if self.loaded_effect.is_queued() {
                self.loaded_effect.state = State::Playing;

                let effect = self.loaded_effect.effect.clone();
                manager.custom_effect(effect);
            }
        }

        self.app_settings.last_profile = Some(self.current_profile.clone());
        self.app_settings.profiles = self.saved_items.profiles.clone();
        self.app_settings.effects = self.saved_items.custom_effects.clone();
        self.app_settings.save();

        self.state_changed = false;
    }

    fn handle_close_request(&mut self, ctx: &Context) {
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        let minimized = ctx.input(|i| i.viewport().minimized.unwrap_or(false));
        
        let just_minimized = minimized && !self.was_minimized;
        self.was_minimized = minimized;

        if (close_requested || just_minimized) && !*DENY_HIDING {
            if self.has_tray.load(Ordering::Relaxed) {
                if close_requested {
                    ctx.send_viewport_cmd(ViewportCommand::CancelClose);
                }
                ctx.send_viewport_cmd(ViewportCommand::Visible(false));
                self.visible.store(false, Ordering::SeqCst);
            }
        }
    }
}

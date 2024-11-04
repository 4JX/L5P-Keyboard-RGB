use std::{mem, process, thread, time::Duration};

#[cfg(target_os = "linux")]
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use device_query::{DeviceQuery, Keycode};
#[cfg(debug_assertions)]
use eframe::egui::style::DebugOptions;
use eframe::{
    egui::{CentralPanel, Context, Frame, Layout, ScrollArea, Style, TopBottomPanel, ViewportCommand},
    emath::Align,
    epaint::{Color32, Rounding, Vec2},
    CreationContext,
};

use strum::IntoEnumIterator;
#[cfg(target_os = "linux")]
use tray_icon::menu::MenuEvent;
#[cfg(not(target_os = "linux"))]
use tray_icon::{menu::MenuEvent, TrayIcon};

use crate::{
    cli::OutputType,
    effects::{self, custom_effect::CustomEffect, EffectManager, ManagerCreationError},
    enums::Effects,
    persist::Settings,
    tray::{self, QUIT_ID, SHOW_ID},
};

use self::{effect_options::EffectOptions, menu_bar::MenuBarState, profile_list::ProfileList, style::Theme};

mod effect_options;
mod menu_bar;
mod modals;
mod profile_list;
mod style;

pub struct App {
    settings: Settings,

    instance_not_unique: bool,
    gui_tx: crossbeam_channel::Sender<GuiMessage>,
    gui_rx: crossbeam_channel::Receiver<GuiMessage>,

    // For Linux
    #[cfg(target_os = "linux")]
    has_tray: Arc<AtomicBool>,
    // The tray struct needs to be kept from being dropped for the tray to appear on windows/mac
    // If this is none it will be assumed there's no tray regardless of cause
    #[cfg(not(target_os = "linux"))]
    tray: Option<TrayIcon>,

    manager: Option<EffectManager>,
    profile_changed: bool,
    custom_effect: CustomEffectState,

    menu_bar: MenuBarState,
    profile_list: ProfileList,
    effect_options: EffectOptions,
    global_rgb: [u8; 3],
    theme: Theme,
}

pub enum GuiMessage {
    CycleProfiles,
    Quit,
}

#[derive(Default)]
pub enum CustomEffectState {
    #[default]
    None,
    Queued(CustomEffect),
    Playing,
}

impl App {
    pub fn new(output: OutputType) -> Self {
        let (gui_tx, gui_rx) = crossbeam_channel::unbounded::<GuiMessage>();

        let manager_result = EffectManager::new(effects::OperationMode::Gui);

        let instance_not_unique = if let Err(err) = &manager_result {
            &ManagerCreationError::InstanceAlreadyRunning == err.current_context()
        } else {
            false
        };

        let manager = manager_result.ok();

        let settings: Settings = Settings::load();
        let profiles = settings.profiles.clone();

        let gui_tx_c = gui_tx.clone();
        // Default app state
        let mut app = Self {
            settings,

            instance_not_unique,
            gui_tx,
            gui_rx,
            #[cfg(target_os = "linux")]
            has_tray: Arc::new(AtomicBool::new(false)),
            #[cfg(not(target_os = "linux"))]
            tray: None,

            manager,
            // Default to true for an instant update on launch
            profile_changed: true,
            custom_effect: CustomEffectState::default(),

            menu_bar: MenuBarState::new(gui_tx_c),
            profile_list: ProfileList::new(profiles),
            effect_options: EffectOptions::default(),
            global_rgb: [0; 3],
            theme: Theme::default(),
        };

        // Update the state according to the option chosen by the user
        match output {
            OutputType::Profile(profile) => app.settings.current_profile = profile,
            OutputType::Custom(effect) => app.custom_effect = CustomEffectState::Queued(effect),
            OutputType::NoArgs => {}
            OutputType::Exit => unreachable!("Exiting the app supersedes starting the GUI"),
        }

        app
    }

    pub fn init(mut self, cc: &CreationContext<'_>, hide_window: bool) -> Self {
        cc.egui_ctx.send_viewport_cmd(ViewportCommand::Visible(!hide_window));

        #[cfg(not(target_os = "linux"))]
        {
            self.tray = tray::build_tray(true);
        }
        // Since egui uses winit under the hood and doesn't use gtk on Linux, and we need gtk for
        // the tray icon to show up, we need to spawn a thread
        // where we initialize gtk and create the tray_icon
        #[cfg(target_os = "linux")]
        {
            let has_tray_c = self.has_tray.clone();

            std::thread::spawn(move || {
                gtk::init().unwrap();

                let tray_icon = tray::build_tray(true);
                has_tray_c.store(tray_icon.is_some(), Ordering::SeqCst);

                gtk::main();
            });
        }

        let egui_ctx = cc.egui_ctx.clone();
        let gui_tx = self.gui_tx.clone();
        std::thread::spawn(move || loop {
            // println!("a");
            // if let Ok(event) = TrayIconEvent::receiver().try_recv() {
            //     println!("{:?}", event);
            // }

            if let Ok(event) = MenuEvent::receiver().recv() {
                if event.id == SHOW_ID {
                    println!("show");
                    egui_ctx.request_repaint();

                    egui_ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                    egui_ctx.send_viewport_cmd(ViewportCommand::Focus);
                } else if event.id == QUIT_ID {
                    println!("quit");
                    egui_ctx.request_repaint();

                    let _ = gui_tx.send(GuiMessage::Quit);
                }
            }
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

        if self.instance_not_unique && modals::unique_instance(ctx) {
            self.exit_app();
        };

        // The uniqueness prompt has priority over generic errors
        if !self.instance_not_unique && self.manager.is_none() && modals::manager_error(ctx) {
            self.exit_app();
        };

        // frame.set_visible(!self.hide_window);

        TopBottomPanel::top("top-panel").show(ctx, |ui| {
            self.menu_bar.show(ctx, ui, &mut self.settings.current_profile, &mut self.custom_effect, &mut self.profile_changed);
        });

        CentralPanel::default()
            .frame(Frame::none().inner_margin(self.theme.spacing.large).fill(Color32::from_gray(26)))
            .show(ctx, |ui| {
                ui.style_mut().spacing.item_spacing = Vec2::splat(self.theme.spacing.large);

                ui.with_layout(Layout::left_to_right(Align::Center).with_cross_justify(true), |ui| {
                    ui.vertical(|ui| {
                        let can_tweak_colors = self.settings.current_profile.effect.takes_color_array() && matches!(self.custom_effect, CustomEffectState::None);

                        let res = ui.add_enabled_ui(can_tweak_colors, |ui| {
                            ui.style_mut().spacing.item_spacing.y = self.theme.spacing.medium;

                            let response = ui.horizontal(|ui| {
                                ui.style_mut().spacing.interact_size = Vec2::splat(60.0);

                                for i in 0..4 {
                                    self.profile_changed |= ui.color_edit_button_srgb(&mut self.settings.current_profile.rgb_zones[i].rgb).changed();
                                }
                            });

                            ui.style_mut().spacing.interact_size = Vec2::new(response.response.rect.width(), 30.0);

                            if ui.color_edit_button_srgb(&mut self.global_rgb).changed() {
                                for i in 0..4 {
                                    self.settings.current_profile.rgb_zones[i].rgb = self.global_rgb;
                                }

                                self.profile_changed = true;
                            };

                            response.response
                        });

                        ui.set_width(res.inner.rect.width());

                        ui.add_enabled_ui(matches!(self.custom_effect, CustomEffectState::None), |ui| {
                            self.effect_options.show(ui, &mut self.settings.current_profile, &mut self.profile_changed, &self.theme.spacing);
                        });

                        self.profile_list
                            .show(ctx, ui, &mut self.settings.current_profile, &self.theme.spacing, &mut self.profile_changed, &mut self.custom_effect);
                    });

                    ui.vertical_centered_justified(|ui| {
                        if matches!(self.custom_effect, CustomEffectState::Playing) && ui.button("Stop custom effect").clicked() {
                            self.custom_effect = CustomEffectState::None;
                            self.profile_changed = true;
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
                                        if ui.selectable_value(&mut self.settings.current_profile.effect, val, text).clicked() {
                                            self.profile_changed = true;
                                            self.custom_effect = CustomEffectState::None;
                                        };
                                    }
                                });
                            });
                        });
                    });
                });
            });

        if self.profile_changed {
            if let Some(manager) = self.manager.as_mut() {
                if matches!(self.custom_effect, CustomEffectState::None) {
                    manager.set_profile(self.settings.current_profile.clone());
                } else if matches!(self.custom_effect, CustomEffectState::Queued(_)) {
                    let state = mem::replace(&mut self.custom_effect, CustomEffectState::Playing);
                    if let CustomEffectState::Queued(effect) = state {
                        manager.custom_effect(effect);
                    }
                }
            }

            self.profile_changed = false;
        }

        if ctx.input(|i| i.viewport().close_requested()) {
            #[cfg(target_os = "linux")]
            if self.has_tray.load(Ordering::Relaxed) && !std::env::var("WAYLAND_DISPLAY").is_ok() {
                ctx.send_viewport_cmd(ViewportCommand::CancelClose);
                ctx.send_viewport_cmd(ViewportCommand::Visible(false));
            } else {
                // Close normally
            }
            #[cfg(not(target_os = "linux"))]
            if self.tray.is_some() {
                ctx.send_viewport_cmd(ViewportCommand::CancelClose);
                ctx.send_viewport_cmd(ViewportCommand::Visible(false));
            } else {
                // Close normally
            }
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.settings.profiles = std::mem::take(&mut self.profile_list.profiles);

        self.settings.save();
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
        let len = self.profile_list.profiles.len();

        let current_profile_name = &self.settings.current_profile.name;

        if let Some((i, _)) = self.profile_list.profiles.iter().enumerate().find(|(_, profile)| &profile.name == current_profile_name) {
            if i == len - 1 && len > 0 {
                self.settings.current_profile = self.profile_list.profiles[0].clone();
            } else {
                self.settings.current_profile = self.profile_list.profiles[i + 1].clone();
            }

            self.profile_changed = true;
        }
    }
}

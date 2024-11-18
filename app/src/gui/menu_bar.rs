use crossbeam_channel::Sender;
use eframe::{
    egui::{self, Context},
    epaint::Vec2,
};
use egui_file::FileDialog;
use egui_notify::Toasts;
use std::{path::PathBuf, time::Duration};

use crate::{gui::modals, manager::custom_effect::CustomEffect, profile::Profile};

use super::{CustomEffectState, GuiMessage};

pub struct MenuBarState {
    // TODO: Re-enable when upstream fixes window visibility
    #[allow(dead_code)]
    gui_sender: Sender<GuiMessage>,
    load_profile_dialog: FileDialog,
    load_effect_dialog: FileDialog,
    save_profile_dialog: FileDialog,
}

impl MenuBarState {
    pub(super) fn new(gui_sender: Sender<GuiMessage>) -> Self {
        Self {
            gui_sender,
            load_profile_dialog: FileDialog::open_file(None).default_size(Vec2::splat(300.0)),
            load_effect_dialog: FileDialog::open_file(None).default_size(Vec2::splat(300.0)),
            save_profile_dialog: FileDialog::save_file(None).default_size(Vec2::splat(300.0)),
        }
    }

    pub fn show(&mut self, ctx: &Context, ui: &mut egui::Ui, current_profile: &mut Profile, current_effect: &mut CustomEffectState, changed: &mut bool, toasts: &mut Toasts) {
        self.show_menu(ctx, ui, toasts);
        self.handle_load_profile(ctx, current_profile, changed, toasts);
        self.handle_save_profile(ctx, current_profile, toasts);
        self.handle_load_effect(ctx, current_effect, changed, toasts);
    }

    fn handle_load_profile(&mut self, ctx: &Context, current_profile: &mut Profile, changed: &mut bool, toasts: &mut Toasts) {
        if self.load_profile_dialog.show(ctx).selected() {
            if let Some(path) = self.load_profile_dialog.path().map(|p| p.to_path_buf()) {
                match Profile::load_profile(&path) {
                    Ok(profile) => {
                        *current_profile = profile;
                        *changed = true;
                    }
                    Err(_) => {
                        toasts.error("Could not load profile.").set_duration(Some(Duration::from_millis(5000))).set_closable(true);
                    }
                }
                self.update_paths(path);
            }
        }
    }

    fn handle_save_profile(&mut self, ctx: &Context, current_profile: &mut Profile, toasts: &mut Toasts) {
        if self.save_profile_dialog.show(ctx).selected() {
            if let Some(path) = self.save_profile_dialog.path().map(|p| p.to_path_buf()) {
                if current_profile.save_profile(&path).is_err() {
                    toasts.error("Could not save profile.").set_duration(Some(Duration::from_millis(5000))).set_closable(true);
                }
                self.update_paths(path);
            }
        }
    }

    fn handle_load_effect(&mut self, ctx: &Context, current_effect: &mut CustomEffectState, changed: &mut bool, toasts: &mut Toasts) {
        if self.load_effect_dialog.show(ctx).selected() {
            if let Some(path) = self.load_effect_dialog.path().map(|p| p.to_path_buf()) {
                match CustomEffect::from_file(&path) {
                    Ok(effect) => {
                        *current_effect = CustomEffectState::Queued(effect);
                        *changed = true;
                    }
                    Err(_) => {
                        toasts.error("Could not load custom effect.").set_duration(Some(Duration::from_millis(5000))).set_closable(true);
                    }
                }
                self.update_paths(path);
            }
        }
    }

    fn update_paths(&mut self, path: PathBuf) {
        let mut save_paths = |path: PathBuf| {
            self.load_profile_dialog.set_path(path.clone());
            self.load_effect_dialog.set_path(path.clone());
            self.save_profile_dialog.set_path(path);
        };

        if path.exists() {
            if path.is_file() {
                if let Some(parent) = path.parent() {
                    save_paths(parent.to_path_buf())
                }
            } else {
                save_paths(path)
            }
        }
    }

    #[allow(unused_variables)]
    fn show_menu(&mut self, ctx: &Context, ui: &mut egui::Ui, toasts: &mut Toasts) {
        use egui::menu;

        menu::bar(ui, |ui| {
            ui.menu_button("Profile", |ui| {
                if ui.button("Open").clicked() {
                    self.load_profile_dialog.open();
                }
                if ui.button("Save").clicked() {
                    self.save_profile_dialog.open();
                }
            });

            ui.menu_button("Effect", |ui| {
                if ui.button("Open").clicked() {
                    self.load_effect_dialog.open();
                }
            });

            let about_modal = modals::about(ctx);
            if ui.button("About").clicked() {
                about_modal.open();
            }

            if ui.button("Donate").clicked() {
                open::that("https://www.buymeacoffee.com/4JXdev").unwrap();
            }

            // if ui.button("Exit").clicked() {
            //     self.gui_sender.send(GuiMessage::Quit).unwrap();
            // }

            #[cfg(target_os = "windows")]
            {
                use crate::console;
                use eframe::{egui::Layout, emath::Align};
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("📜").clicked() {
                        if !console::alloc_with_color_support() {
                            toasts.error("Could not allocate debug terminal.").set_duration(Some(Duration::from_millis(5000))).set_closable(true);
                        }
                        println!("Debug terminal enabled.");
                    }
                });
            }
        });
    }
}

use crossbeam_channel::Sender;
use eframe::{
    egui::{self, Context},
    epaint::Vec2,
};
use egui_file::FileDialog;
use egui_notify::Toasts;
use std::{path::PathBuf, time::Duration};

use crate::{effects::custom_effect::CustomEffect, gui::modals, profile::Profile};

use super::{CustomEffectState, GuiMessage};

pub struct MenuBarState {
    gui_sender: Sender<GuiMessage>,
    opened_file: Option<PathBuf>,
    open_file_dialog: Option<FileDialog>,
    file_kind: Option<FileOperation>,
    toasts: Toasts,
}

impl MenuBarState {
    pub(super) fn new(gui_sender: Sender<GuiMessage>) -> Self {
        Self {
            gui_sender,
            opened_file: None,
            open_file_dialog: None,
            file_kind: None,
            toasts: Toasts::default(),
        }
    }
}

enum FileOperation {
    LoadProfile,
    LoadEffect,
    SaveProfile,
}

impl MenuBarState {
    pub fn show(&mut self, ctx: &Context, ui: &mut egui::Ui, current_profile: &mut Profile, current_effect: &mut CustomEffectState, changed: &mut bool) {
        self.toasts.show(ctx);

        self.show_menu(ctx, ui);

        if let Some(dialog) = &mut self.open_file_dialog {
            if dialog.show(ctx).selected() {
                if let Some(path) = dialog.path() {
                    self.opened_file = Some(path.to_owned());
                    if let Some(kind) = &self.file_kind {
                        match kind {
                            FileOperation::LoadProfile => match Profile::load_profile(path) {
                                Ok(profile) => {
                                    *current_profile = profile;
                                    *changed = true;
                                }
                                Err(_) => {
                                    self.toasts.error("Could not load profile.").set_duration(Some(Duration::from_millis(5000))).set_closable(true);
                                }
                            },
                            FileOperation::LoadEffect => match CustomEffect::from_file(path) {
                                Ok(effect) => {
                                    *current_effect = CustomEffectState::Queued(effect);
                                    *changed = true;
                                }
                                Err(_) => {
                                    self.toasts.error("Could not load custom effect.").set_duration(Some(Duration::from_millis(5000))).set_closable(true);
                                }
                            },
                            FileOperation::SaveProfile => {
                                if current_profile.save_profile(path).is_err() {
                                    self.toasts.error("Could not save profile.").set_duration(Some(Duration::from_millis(5000))).set_closable(true);
                                };
                            }
                        }
                    }

                    self.file_kind = None;
                }
            }
        }
    }

    fn show_menu(&mut self, ctx: &Context, ui: &mut egui::Ui) {
        use egui::menu;

        menu::bar(ui, |ui| {
            ui.menu_button("Profile", |ui| {
                if ui.button("Open").clicked() {
                    let mut dialog = self.new_open_dialog();
                    self.file_kind = Some(FileOperation::LoadProfile);
                    dialog.open();
                    self.open_file_dialog = Some(dialog);
                }
                if ui.button("Save").clicked() {
                    let mut dialog = self.new_save_dialog();
                    self.file_kind = Some(FileOperation::SaveProfile);
                    dialog.open();
                    self.open_file_dialog = Some(dialog);
                }
            });

            ui.menu_button("Effect", |ui| {
                if ui.button("Open").clicked() {
                    let mut dialog = self.new_open_dialog();
                    self.file_kind = Some(FileOperation::LoadEffect);
                    dialog.open();
                    self.open_file_dialog = Some(dialog);
                }
            });

            let about_modal = modals::about(ctx);
            if ui.button("About").clicked() {
                about_modal.open();
            }

            if ui.button("Donate").clicked() {
                open::that("https://www.buymeacoffee.com/4JXdev").unwrap();
            }

            if ui.button("Exit").clicked() {
                self.gui_sender.send(GuiMessage::Quit).unwrap();
            }
        });
    }

    fn new_open_dialog(&self) -> FileDialog {
        FileDialog::open_file(self.opened_file.clone()).default_size(Vec2::splat(300.0))
    }

    fn new_save_dialog(&self) -> FileDialog {
        FileDialog::save_file(self.opened_file.clone()).default_size(Vec2::splat(300.0))
    }
}

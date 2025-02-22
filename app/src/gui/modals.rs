use eframe::egui::{Color32, Context, Frame, ScrollArea};
use egui_modal::Modal;

use crate::util;

pub fn unique_instance(ctx: &Context) -> bool {
    let mut exit_app = false;

    let modal = Modal::new(ctx, "unique_instance_error_modal");

    modal.show(|ui| {
        modal.title(ui, "Warning");
        modal.frame(ui, |ui| {
            modal.body(ui, "Another instance is already running, please close it and try again.");
        });

        modal.buttons(ui, |ui| {
            exit_app = modal.caution_button(ui, "Exit").clicked();
        });
    });

    modal.open();

    exit_app
}

pub fn manager_error(ctx: &Context) -> bool {
    let mut exit_app = false;

    let modal = Modal::new(ctx, "manager_error_modal");

    modal.show(|ui| {
        modal.title(ui, "Warning");
        modal.frame(ui, |ui| {
            modal.body(ui, "Failed to find a valid keyboard.");
            modal.body(ui, "Ensure that you have a supported model and that the application has access to it.");
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = -2.0;

                modal.body(ui, "On Linux, see");
                util::clickable_link(ui, "https://github.com/4JX/L5P-Keyboard-RGB#usage");
            });
            modal.body(ui, "In certain cases, this may be due to a hardware error.");

            if let Ok(list) = legion_rgb_driver::find_possible_keyboards() {
                modal.body(ui, "Please attach the following list of identifiers when making an issue:");
                Frame::new().fill(Color32::from_gray(20)).inner_margin(5.0).corner_radius(6.0).show(ui, |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        if list.is_empty() {
                            ui.label("No candidates found");
                        } else {
                            for d in list {
                                ui.label(d);
                            }
                        }
                    });
                });
            }
        });

        modal.buttons(ui, |ui| {
            exit_app = modal.caution_button(ui, "Exit").clicked();
        });
    });

    modal.open();

    exit_app
}

pub fn about(ctx: &Context) -> Modal {
    let modal = Modal::new(ctx, "about_modal");

    modal.show(|ui| {
        modal.title(ui, "About");
        modal.frame(ui, |ui| {
            modal.body(ui, "A program made by 4JX.");
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = -2.0;

                modal.body(ui, "Something's not working?:");
                util::clickable_link(ui, "https://github.com/4JX/L5P-Keyboard-RGB");
            });

            modal.body(ui, format!("Current version: {}", env!("CARGO_PKG_VERSION")));
        });

        modal.buttons(ui, |ui| modal.button(ui, "Close"));
    });

    modal
}

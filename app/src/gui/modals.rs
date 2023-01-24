use eframe::egui::Context;
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

pub fn update_available(ctx: &Context, new_version: &str, skip_version: &mut bool, show_modal: &mut bool) {
    let modal = Modal::new(ctx, "manager_error_modal");

    modal.show(|ui| {
        modal.title(ui, "An update is available!");
        modal.frame(ui, |ui| {
            modal.body(ui, format!("Current version: {}", env!("CARGO_PKG_VERSION")));
            modal.body(ui, format!("New version: {}", new_version));

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = -2.0;

                modal.body(ui, "Download:");
                util::clickable_link(ui, "https://github.com/4JX/L5P-Keyboard-RGB/releases");
            });
        });

        modal.buttons(ui, |ui| {
            *show_modal = !modal.button(ui, "Close").clicked();

            if modal.button(ui, "Skip version").clicked() {
                *skip_version = true;
                *show_modal = false;
            }
        });
    });

    modal.open()
}

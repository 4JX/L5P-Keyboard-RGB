use eframe::{
    egui::{Context, Frame, ScrollArea, Ui},
    epaint::{Color32, Rounding},
};
use egui_modal::Modal;

use crate::profile::Profile;

use super::{style::SpacingStyle, CustomEffectState};

pub struct ProfileList {
    pub profiles: Vec<Profile>,
    new_profile_name: String,
}

impl ProfileList {
    pub fn new(profiles: Vec<Profile>) -> Self {
        Self {
            profiles,
            new_profile_name: String::default(),
        }
    }
}

impl ProfileList {
    pub fn show(&mut self, ctx: &Context, ui: &mut Ui, current_profile: &mut Profile, spacing: &SpacingStyle, changed: &mut bool, custom_effect_state: &mut CustomEffectState) {
        ui.scope(|ui| {
            let modal = Modal::new(ctx, "profile_name_modal");

            modal.show(|ui| {
                modal.title(ui, "Enter the name of the profile");
                modal.frame(ui, |ui| {
                    ui.text_edit_singleline(&mut self.new_profile_name);
                });

                modal.buttons(ui, |ui| {
                    let is_empty = self.new_profile_name.is_empty();
                    let name_not_unique = self.profiles.iter().any(|prof| prof.name.as_ref() == Some(&self.new_profile_name));

                    modal.button(ui, "Cancel");

                    ui.add_enabled_ui(!is_empty && !name_not_unique, |ui| {
                        if modal.button(ui, "Save").clicked() {
                            current_profile.name = Some(self.new_profile_name.clone());
                            self.profiles.push(current_profile.clone());
                        };
                    });

                    if is_empty {
                        ui.label("You must enter a name");
                    } else if name_not_unique {
                        ui.label("Name already in use");
                    }
                });
            });

            ui.style_mut().spacing.item_spacing = spacing.default;

            ui.horizontal(|ui| {
                ui.heading("Profiles");
                if ui.button("+").clicked() {
                    self.new_profile_name.clear();

                    modal.open();

                    // let original_name = profile.name.clone();
                    // let mut name = original_name.clone();
                    // let mut i = 1;

                    // // Find an unique name
                    // while self.profiles.iter().any(|prof| prof.name == name) {
                    // 	i += 1;
                    // 	name = format!("{original_name} ({i})");
                    // }

                    // // If the name had to be changed, update the profile
                    // if original_name != name {
                    // 	profile.name = name;
                    // }

                    // self.profiles.push(profile.clone());
                }
                if ui.button("-").clicked() {
                    self.profiles.retain(|prof| prof != current_profile);
                }
            });

            Frame {
                rounding: Rounding::same(6.0),
                fill: Color32::from_gray(20),
                ..Frame::default()
            }
            .show(ui, |ui| {
                ui.set_height(ui.available_height());

                if self.profiles.is_empty() {
                    ui.centered_and_justified(|ui| ui.label("No profiles added"));
                } else {
                    ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            for prof in &self.profiles {
                                let name = prof.name.as_ref().map(|s| s.as_str()).unwrap_or("Unnamed");
                                if ui.selectable_value(current_profile, prof.clone(), name).clicked() {
                                    *changed = true;
                                    *custom_effect_state = CustomEffectState::None;
                                };
                            }
                        });
                    });
                }
            });
        });
    }
}

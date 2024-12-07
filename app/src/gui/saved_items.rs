use eframe::{
    egui::{Context, Frame, RichText, ScrollArea, Ui},
    epaint::{Color32, Rounding},
};
use egui_modal::Modal;

use crate::manager::{custom_effect::CustomEffect, profile::Profile};

use super::{style::SpacingStyle, LoadedEffect, State};

#[derive(Clone)]
pub struct SavedItems {
    pub custom_effects: Vec<CustomEffect>,
    pub profiles: Vec<Profile>,

    tab: Tab,
    new_item_name: String,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Tab {
    Profiles,
    CustomEffects,
}

impl SavedItems {
    pub fn new(profiles: Vec<Profile>, custom_effects: Vec<CustomEffect>) -> Self {
        Self {
            profiles,
            custom_effects,
            tab: Tab::Profiles,
            new_item_name: String::default(),
        }
    }

    fn setup_modal<T: Clone>(
        ctx: &Context, id_source: &str, item_name: &str, new_item_name: &mut String, items: &mut Vec<T>, current_item: &mut T, item_name_extractor: fn(&T) -> Option<String>,
        item_name_setter: fn(&mut T, String),
    ) -> Modal {
        let modal = Modal::new(ctx, id_source);

        modal.show(|ui| {
            modal.title(ui, item_name);
            modal.frame(ui, |ui| {
                ui.text_edit_singleline(new_item_name);
            });

            modal.buttons(ui, |ui| {
                let is_empty = new_item_name.is_empty();
                let name_not_unique = items.iter().any(|item| item_name_extractor(item) == Some(new_item_name.clone()));

                modal.button(ui, "Cancel");

                ui.add_enabled_ui(!is_empty && !name_not_unique, |ui| {
                    if modal.button(ui, "Save").clicked() {
                        item_name_setter(current_item, new_item_name.clone());
                        items.push(current_item.clone());
                    };
                });

                if is_empty {
                    ui.label("You must enter a name");
                } else if name_not_unique {
                    ui.label("Name already in use");
                }
            });
        });

        modal
    }

    pub fn setup_profile_modal(&mut self, ctx: &Context, current_profile: &mut Profile) -> Modal {
        Self::setup_modal(
            ctx,
            "profile_modal",
            "Enter the name of the profile",
            &mut self.new_item_name,
            &mut self.profiles,
            current_profile,
            |prof| prof.name.clone(),
            |prof, name| prof.name = Some(name),
        )
    }

    pub fn setup_effect_modal(&mut self, ctx: &Context, loaded_effect: &mut LoadedEffect) -> Modal {
        Self::setup_modal(
            ctx,
            "effect_modal",
            "Enter the name of the custom effect",
            &mut self.new_item_name,
            &mut self.custom_effects,
            &mut loaded_effect.effect,
            |effect| effect.name.clone(),
            |effect, name| effect.name = Some(name),
        )
    }

    pub fn show_header(&mut self, ctx: &Context, ui: &mut Ui, current_profile: &mut Profile, loaded_effect: &mut LoadedEffect) {
        ui.selectable_value(&mut self.tab, Tab::Profiles, RichText::new("Profiles").heading());
        ui.selectable_value(&mut self.tab, Tab::CustomEffects, RichText::new("Custom Effects").heading());

        let profile_modal = self.setup_profile_modal(ctx, current_profile);
        let effect_modal = self.setup_effect_modal(ctx, loaded_effect);

        match self.tab {
            Tab::Profiles => {
                if ui.button("+").clicked() {
                    self.new_item_name.clear();
                    profile_modal.open();
                }
                if ui.button("-").clicked() {
                    self.profiles.retain(|prof| prof != current_profile);
                }
            }
            Tab::CustomEffects => {
                if loaded_effect.is_playing() {
                    if ui.button("+").clicked() {
                        self.new_item_name.clear();
                        effect_modal.open();
                    }
                }

                if ui.button("-").clicked() {
                    self.custom_effects.retain(|effect| effect != &loaded_effect.effect);
                }
            }
        }
    }

    pub fn show(&mut self, ctx: &Context, ui: &mut Ui, current_profile: &mut Profile, loaded_effect: &mut LoadedEffect, spacing: &SpacingStyle, changed: &mut bool) {
        ui.scope(|ui: &mut Ui| {
            ui.style_mut().spacing.item_spacing = spacing.default;

            ui.horizontal(|ui| {
                self.show_header(ctx, ui, current_profile, loaded_effect);
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
                        ui.horizontal_wrapped(|ui| match self.tab {
                            Tab::Profiles => {
                                for prof in self.profiles.iter() {
                                    let name = prof.name.as_deref().unwrap_or("Unnamed");
                                    if ui.selectable_value(current_profile, prof.clone(), name).clicked() {
                                        *changed = true;
                                        loaded_effect.state = State::None;
                                    };
                                }
                            }
                            Tab::CustomEffects => {
                                for effect in self.custom_effects.iter() {
                                    let name = effect.name.as_deref().unwrap_or("Unnamed");
                                    if ui.selectable_value(&mut loaded_effect.effect, effect.clone(), name).clicked() {
                                        *changed = true;
                                        loaded_effect.effect = effect.clone();
                                        loaded_effect.state = State::Queued;
                                    };
                                }
                            }
                        });
                    });
                }
            });
        });
    }
}

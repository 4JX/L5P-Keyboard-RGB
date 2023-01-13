use eframe::{
    egui::{style::Selection, Visuals},
    epaint::{Color32, Stroke, Vec2},
};

pub struct Theme {
    pub spacing: SpacingStyle,
    pub visuals: Visuals,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            spacing: Default::default(),
            visuals: Visuals {
                selection: Selection {
                    bg_fill: Color32::from_rgb(40, 100, 160),
                    stroke: Stroke::new(1.0, Color32::from_rgb(192, 222, 255)),
                },
                ..Visuals::default()
            },
        }
    }
}

pub struct SpacingStyle {
    pub default: Vec2,
    pub large: f32,
    pub medium: f32,
}

impl Default for SpacingStyle {
    fn default() -> Self {
        Self {
            default: Vec2::new(8.0, 5.0),
            large: 20.0,
            medium: 8.0,
        }
    }
}

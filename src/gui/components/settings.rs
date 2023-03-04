use egui::{Button, Checkbox, Pos2, Slider, Window};

use super::GuiComponent;
use crate::gui::{GuiEvent, GuiEvents};
use crate::utils::constants::{
    ANIMATION_ACTIVE_BY_DEFAULT, DEFAULT_LIGHT_COLOR, DEFAULT_LIGHT_DIRECTION,
    DEFAULT_PROBE_RADIUS, DEFAULT_SES_RESOLUTION, MAX_PROBE_RADIUS, MAX_SES_RESOLUTION,
};

pub struct UserSettings {
    ses_resolution: u32,
    probe_radius: f32,
    render_spacefill: bool,
    render_ses: bool,

    direction: [f32; 3],
    light_color: [f32; 3],

    is_animation_active: bool,
    animation_speed: u32,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            ses_resolution: DEFAULT_SES_RESOLUTION,
            probe_radius: DEFAULT_PROBE_RADIUS,
            render_spacefill: true,
            render_ses: true,

            direction: DEFAULT_LIGHT_DIRECTION,
            light_color: DEFAULT_LIGHT_COLOR,

            is_animation_active: ANIMATION_ACTIVE_BY_DEFAULT,
            animation_speed: 5,
        }
    }
}

impl GuiComponent for UserSettings {
    #[rustfmt::skip]
    fn draw(&mut self, context: &egui::Context, events: &mut GuiEvents) {
        let window = Window::new("Settings").default_pos(Pos2::new(100.0, 100.0)).default_width(100.0);

        window.show(context, |ui| {
            // Model parameters.
            if ui.add(self.ses_slider()).changed() {
                events.push(GuiEvent::SesResolutionChanged(self.ses_resolution));
            }
            if ui.add(self.probe_slider()).changed() {
                events.push(GuiEvent::ProbeRadiusChanged(self.probe_radius));
            }
            ui.separator();

            ui.collapsing("Render Passes", |ui| {
                if ui.add(self.render_spacefill_checkbox()).changed() {
                    events.push(GuiEvent::RenderSpacefillChanged(self.render_spacefill));
                }
                if ui.add(self.render_ses_checkbox()).changed() {
                    events.push(GuiEvent::RenderSesChanged(self.render_ses));
                }
            });

            // Light options.
            ui.collapsing("Lighting", |ui| {
                ui.label("Light direction:");
                for (i, coord) in ["X", "Y", "Z"].iter().enumerate() {
                    if ui.add(Slider::new(&mut self.direction[i], -1.0..=1.0).text(*coord)).changed() {
                        events.push(GuiEvent::UpdateLight((self.direction, self.light_color)));
                    }
                }

                ui.label("Light color:");
                if ui.color_edit_button_rgb(&mut self.light_color).changed() {
                    events.push(GuiEvent::UpdateLight((self.direction, self.light_color)));}
            });


            // Animation.
            ui.collapsing("Animation", |ui| {
                if ui.add(Slider::new(&mut self.animation_speed, 1..=10).text("Speed")).changed() {
                    events.push(GuiEvent::AnimationSpeedChanged(self.animation_speed));
                }
                ui.horizontal(|ui| {
                    if ui.add(self.toggle_animation_button()).clicked() {
                        events.push(GuiEvent::ToggleAnimation);
                        self.is_animation_active = !self.is_animation_active;
                    }
                    match self.is_animation_active {
                        true => ui.label("Playing ✅"),
                        false => ui.label("Paused ❌"),
                    };
                });
            });

        });
    }

    fn should_close(&self) -> bool {
        false
    }
}

impl UserSettings {
    fn ses_slider(&mut self) -> Slider {
        Slider::new(
            &mut self.ses_resolution,
            DEFAULT_SES_RESOLUTION..=MAX_SES_RESOLUTION,
        )
        .text("SES resolution")
    }
    fn probe_slider(&mut self) -> Slider {
        Slider::new(
            &mut self.probe_radius,
            DEFAULT_PROBE_RADIUS..=MAX_PROBE_RADIUS,
        )
        .text("Probe radius")
    }
    fn render_spacefill_checkbox(&mut self) -> Checkbox {
        Checkbox::new(&mut self.render_spacefill, "Spacefill")
    }
    fn render_ses_checkbox(&mut self) -> Checkbox {
        Checkbox::new(&mut self.render_ses, "SES")
    }
    fn toggle_animation_button(&mut self) -> Button {
        match self.is_animation_active {
            true => Button::new("⏸"),
            false => Button::new("▶"),
        }
    }
}

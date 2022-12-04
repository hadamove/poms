use egui::{Button, Checkbox, Pos2, Slider, Window};

use super::GuiComponent;
use crate::gui::{GuiEvent, GuiEvents};
use crate::utils::constants::{DEFAULT_PROBE_RADIUS, DEFAULT_SES_RESOLUTION};

pub struct UserSettings {
    ses_resolution: u32,
    probe_radius: f32,
    render_spacefill: bool,
    render_ses: bool,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            ses_resolution: DEFAULT_SES_RESOLUTION,
            probe_radius: DEFAULT_PROBE_RADIUS,
            render_spacefill: true,
            render_ses: true,
        }
    }
}

impl GuiComponent for UserSettings {
    #[rustfmt::skip]
    fn draw(&mut self, context: &egui::Context, events: &mut GuiEvents) {
        let window = Window::new("Settings").default_pos(Pos2::new(100.0, 100.0));

        window.show(context, |ui| {
            // Model parameters.
            if ui.add(self.ses_slider()).changed() {
                events.push(GuiEvent::SesResolutionChanged(self.ses_resolution));
            }
            if ui.add(self.probe_slider()).changed() {
                events.push(GuiEvent::ProbeRadiusChanged(self.probe_radius));
            }
            ui.separator();

            // Render options.
            if ui.add(self.render_spacefill_checkbox()).changed() {
                events.push(GuiEvent::RenderSpacefillChanged(self.render_spacefill));
            }
            if ui.add(self.render_ses_checkbox()).changed() {
                events.push(GuiEvent::RenderSesChanged(self.render_ses));
            }

            ui.separator();
            if ui.add(self.toggle_animation_button()).clicked() {
                events.push(GuiEvent::ToggleAnimation);
            }
        });
    }

    fn should_close(&self) -> bool {
        false
    }
}

impl UserSettings {
    fn ses_slider(&mut self) -> Slider {
        Slider::new(&mut self.ses_resolution, 64..=256).text("SES resolution")
    }
    fn probe_slider(&mut self) -> Slider {
        Slider::new(&mut self.probe_radius, 1.0..=5.0).text("Probe radius")
    }
    fn render_spacefill_checkbox(&mut self) -> Checkbox {
        Checkbox::new(&mut self.render_spacefill, "Render spacefill")
    }
    fn render_ses_checkbox(&mut self) -> Checkbox {
        Checkbox::new(&mut self.render_ses, "Render SES surface")
    }
    fn toggle_animation_button(&mut self) -> Button {
        Button::new("Play/Pause")
    }
}

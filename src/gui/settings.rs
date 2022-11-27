use egui::{Checkbox, Pos2, Slider, Window};

use super::GuiComponent;
use crate::shared::events::{AppEvent, EventDispatch};

pub struct UserSettings {
    pub ses_resolution: u32,
    pub probe_radius: f32,

    pub render_spacefill: bool,
    pub render_ses: bool,

    pub compute_ses_always: bool,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            // TODO: use well defined constants.
            ses_resolution: 64,
            probe_radius: 1.4,

            render_spacefill: true,
            render_ses: true,

            compute_ses_always: false,
        }
    }
}

impl GuiComponent for UserSettings {
    #[rustfmt::skip]
    fn draw(&mut self, context: &egui::Context, dispatch: &EventDispatch) {
        let window = Window::new("Settings").default_pos(Pos2::new(100.0, 100.0));
        window.show(context, |ui| {
            // Model parameters.
            if ui.add(self.ses_slider()).changed() {
                dispatch.send(AppEvent::SesResolutionChanged(self.ses_resolution)).ok();
            }
            if ui.add(self.probe_slider()).changed() {
                dispatch.send(AppEvent::ProbeRadiusChanged(self.probe_radius)).ok();
            }
            ui.separator();

            // Render options.
            if ui.add(self.render_spacefill_checkbox()).changed() {
                dispatch.send(AppEvent::RenderSpacefillChanged(self.render_spacefill)).ok();
            }
            if ui.add(self.render_ses_checkbox()).changed() {
                dispatch.send(AppEvent::RenderSesChanged(self.render_ses)).ok();
            }
            ui.separator();

            // Compute options.
            if ui.add(self.compute_ses_always_checkbox()).changed() {
                dispatch.send(AppEvent::ComputeSesAlwaysChanged(self.compute_ses_always)).ok();
            }
        });
    }

    fn update(&mut self, _event: &AppEvent) {}
}

impl UserSettings {
    fn ses_slider(&mut self) -> Slider {
        Slider::new(&mut self.ses_resolution, 8..=128).text("SES resolution")
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
    fn compute_ses_always_checkbox(&mut self) -> Checkbox {
        Checkbox::new(&mut self.compute_ses_always, "Compute SES always")
    }
}

use egui::{Button, Checkbox, Pos2, Slider, Window};

use crate::app::limits::{MAX_DISTANCE_FIELD_RESOLUTION, MAX_PROBE_RADIUS, MIN_PROBE_RADIUS};
use crate::app::ui::{events::UserEvent, UIState};

/// Component that displays settings window.
/// Allows to change model parameters and toggle render passes.
pub fn settings(context: &egui::Context, state: &mut UIState) {
    let window = Window::new("Settings")
        .default_pos(Pos2::new(100.0, 100.0))
        .default_width(100.0);

    window.show(context, |ui| {
        // Model parameters.
        if ui.add(resolution_slider(state)).changed() {
            state.dispatch_event(UserEvent::DistanceFieldResolutionChanged {
                resolution: state.target_resolution,
            });
        }
        if ui.add(probe_radius_slider(state)).changed() {
            state.dispatch_event(UserEvent::ProbeRadiusChanged {
                probe_radius: state.probe_radius,
            });
        }
        ui.separator();

        ui.collapsing("Render Passes", |ui| {
            if ui.add(spacefill_pass_checkbox(state)).changed() {
                state.dispatch_event(UserEvent::RenderSpacefillChanged {
                    is_enabled: state.render_spacefill,
                });
            }
            if ui.add(molecular_surface_pass_checkbox(state)).changed() {
                state.dispatch_event(UserEvent::RenderMolecularSurfaceChanged {
                    is_enabled: state.render_molecular_surface,
                });
            }
        });

        // Animation.
        ui.collapsing("Animation", |ui| {
            if ui
                .add(Slider::new(&mut state.animation_speed, 1..=10).text("Speed"))
                .changed()
            {
                state.dispatch_event(UserEvent::AnimationSpeedChanged {
                    speed: state.animation_speed,
                });
            }
            ui.horizontal(|ui| {
                if ui.add(animation_button(state)).clicked() {
                    state.dispatch_event(UserEvent::ToggleAnimation);
                    state.is_animation_active = !state.is_animation_active;
                }
                match state.is_animation_active {
                    true => ui.label("Playing ✅"),
                    false => ui.label("Paused ❌"),
                };
            });
        });
    });
}

fn resolution_slider(state: &mut UIState) -> Slider {
    Slider::new(
        &mut state.target_resolution,
        64..=MAX_DISTANCE_FIELD_RESOLUTION,
    )
    .text("SES resolution")
}

fn probe_radius_slider(state: &mut UIState) -> Slider {
    Slider::new(&mut state.probe_radius, MIN_PROBE_RADIUS..=MAX_PROBE_RADIUS).text("Probe radius")
}

fn spacefill_pass_checkbox(state: &mut UIState) -> Checkbox {
    Checkbox::new(&mut state.render_spacefill, "Spacefill")
}

fn molecular_surface_pass_checkbox(state: &mut UIState) -> Checkbox {
    Checkbox::new(&mut state.render_molecular_surface, "Molecular Surface")
}

fn animation_button(state: &mut UIState) -> Button {
    match state.is_animation_active {
        true => Button::new("⏸"),
        false => Button::new("▶"),
    }
}

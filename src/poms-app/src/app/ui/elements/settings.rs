use egui::{Button, Checkbox, Pos2, Slider, Widget, Window};
use poms_common::limits::{
    MAX_ANIMATION_SPEED, MAX_DISTANCE_FIELD_RESOLUTION, MAX_PROBE_RADIUS, MIN_ANIMATION_SPEED,
    MIN_DISTANCE_FIELD_RESOLUTION, MIN_PROBE_RADIUS,
};

use crate::app::ui::{events::UserEvent, UIState};

/// Component that displays settings window.
/// Allows to change model parameters and toggle render passes.
pub(crate) fn settings(context: &mut egui::Context, state: &mut UIState) {
    Window::new("Settings")
        .default_pos(Pos2::new(16.0, 36.0))
        .pivot(egui::Align2::LEFT_TOP)
        .max_width(256.)
        // .min_width(400.)
        .resizable(false)
        .show(context, |ui| {
            egui::CollapsingHeader::new("Render Passes")
                .default_open(true)
                .show(ui, |ui| {
                    spacefill_pass_render_settings(ui, state);
                    ui.separator();
                    molecular_surface_render_settings(ui, state);
                    ui.separator();
                    ssao_render_settings(ui, state);
                    ui.separator();
                });

            animation_controls(ui, state);
        });
}

fn spacefill_pass_render_settings(ui: &mut egui::Ui, state: &mut UIState) {
    egui::CollapsingHeader::new("Spacefill")
        .default_open(true)
        .show(ui, |ui| {
            if ui
                .add(Checkbox::new(&mut state.render_spacefill, "Enabled"))
                .changed()
            {
                state.dispatch_event(UserEvent::ChangeRenderSpacefill {
                    is_enabled: state.render_spacefill,
                });
            }
        });
}

fn molecular_surface_render_settings(ui: &mut egui::Ui, state: &mut UIState) {
    egui::CollapsingHeader::new("Molecular Surface")
        .default_open(true)
        .show(ui, |ui| {
            // Enable/disable molecular surface checkbox.
            if ui
                .add(Checkbox::new(
                    &mut state.render_molecular_surface,
                    "Enabled",
                ))
                .changed()
            {
                state.dispatch_event(UserEvent::ChangeRenderMolecularSurface {
                    is_enabled: state.render_molecular_surface,
                });
            }

            // Add molecular surface settings if molecular surface is enabled.
            ui.add_enabled_ui(state.render_molecular_surface, |ui| {
                // Molecular surface resolution slider.
                if ui
                    .add(
                        Slider::new(
                            &mut state.target_resolution,
                            MIN_DISTANCE_FIELD_RESOLUTION..=MAX_DISTANCE_FIELD_RESOLUTION,
                        )
                        .text("Resolution"),
                    )
                    .changed()
                {
                    state.dispatch_event(UserEvent::ChangeDistanceFieldResolution {
                        resolution: state.target_resolution,
                    });
                }

                // Probe radius slider.
                if ui
                    .add(
                        Slider::new(&mut state.probe_radius, MIN_PROBE_RADIUS..=MAX_PROBE_RADIUS)
                            .text("Probe radius"),
                    )
                    .changed()
                {
                    state.dispatch_event(UserEvent::ChangeProbeRadius {
                        probe_radius: state.probe_radius,
                    });
                }

                if let Some(compute_progress) = &state.compute_progress {
                    egui::widgets::ProgressBar::new(compute_progress.progress)
                        .text(format!(
                            "computing {} / {}",
                            compute_progress.current_resolution, compute_progress.target_resolution
                        ))
                        .ui(ui);
                }
            });
        });
}

fn ssao_render_settings(ui: &mut egui::Ui, state: &mut UIState) {
    egui::CollapsingHeader::new("SSAO")
        .default_open(true)
        .show(ui, |ui| {
            let dispatch_settings_changed = |state: &mut UIState| {
                state.dispatch_event(UserEvent::UpdatePostprocessSettings {
                    settings: state.postprocess_settings,
                });
            };

            // Enable/disable SSAO checkbox.
            if ui
                .add(Checkbox::new(
                    &mut state.postprocess_settings.is_ssao_enabled,
                    "Enabled",
                ))
                .changed()
            {
                dispatch_settings_changed(state);
            }

            // Add SSAO settings if SSAO is enabled.
            ui.add_enabled_ui(state.postprocess_settings.is_ssao_enabled, |ui| {
                // Radius slider.
                if ui
                    .add(
                        Slider::new(&mut state.postprocess_settings.ssao_radius, 0.0..=10.0)
                            .text("Radius"),
                    )
                    .changed()
                {
                    dispatch_settings_changed(state);
                }

                // Bias slider.
                if ui
                    .add(
                        Slider::new(&mut state.postprocess_settings.ssao_bias, 0.0..=10.0)
                            .text("Bias"),
                    )
                    .changed()
                {
                    dispatch_settings_changed(state);
                }

                // Number of samples slider.
                if ui
                    .add(
                        Slider::new(&mut state.postprocess_settings.ssao_samples_count, 1..=64)
                            .text("Samples count"),
                    )
                    .changed()
                {
                    dispatch_settings_changed(state);
                }

                // Blur checkbox.
                if ui
                    .add(Checkbox::new(
                        &mut state.postprocess_settings.ssao_is_blur_enabled,
                        "Blur",
                    ))
                    .changed()
                {
                    dispatch_settings_changed(state);
                }
            });
        });
}

fn animation_controls(ui: &mut egui::Ui, state: &mut UIState) {
    egui::CollapsingHeader::new("Animation")
        .default_open(false)
        .show(ui, |ui| {
            if ui
                .add(
                    Slider::new(
                        &mut state.animation_speed,
                        MIN_ANIMATION_SPEED..=MAX_ANIMATION_SPEED,
                    )
                    .text("Speed"),
                )
                .changed()
            {
                state.dispatch_event(UserEvent::ChangeAnimationSpeed {
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
}

fn animation_button(state: &mut UIState) -> Button {
    match state.is_animation_active {
        true => Button::new("⏸"),
        false => Button::new("▶"),
    }
}

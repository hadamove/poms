use egui::{Button, Checkbox, Pos2, Slider, Window};

use crate::app::{
    constants::{
        ANIMATION_ACTIVE_BY_DEFAULT, DEFAULT_DISTANCE_FIELD_RESOLUTION, DEFAULT_PROBE_RADIUS,
        MAX_DISTANCE_FIELD_RESOLUTION, MAX_PROBE_RADIUS,
    },
    ui::event::UserEvent,
};

// TODO: Move this up in the module hierarchy.
pub struct UIState {
    df_resolution: u32,
    probe_radius: f32,
    render_spacefill: bool,
    render_molecular_surface: bool,

    is_animation_active: bool,
    animation_speed: u32,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            df_resolution: DEFAULT_DISTANCE_FIELD_RESOLUTION,
            probe_radius: DEFAULT_PROBE_RADIUS,
            render_spacefill: true,
            render_molecular_surface: true,

            is_animation_active: ANIMATION_ACTIVE_BY_DEFAULT,
            animation_speed: 5,
        }
    }
}

// TODO: This has to be refactored
pub fn settings(context: &egui::Context, state: &mut UIState, dispatch: &mut dyn FnMut(UserEvent)) {
    let window = Window::new("Settings")
        .default_pos(Pos2::new(100.0, 100.0))
        .default_width(100.0);

    window.show(context, |ui| {
        // Model parameters.
        if ui.add(resolution_slider(state)).changed() {
            dispatch(UserEvent::DistanceFieldResolutionChanged(
                state.df_resolution,
            ));
        }
        if ui.add(probe_radius_slider(state)).changed() {
            dispatch(UserEvent::ProbeRadiusChanged(state.probe_radius));
        }
        ui.separator();

        ui.collapsing("Render Passes", |ui| {
            if ui.add(spacefill_pass_checkbox(state)).changed() {
                dispatch(UserEvent::RenderSpacefillChanged(state.render_spacefill));
            }
            if ui.add(molecular_surface_pass_checkbox(state)).changed() {
                dispatch(UserEvent::RenderMolecularSurfaceChanged(
                    state.render_molecular_surface,
                ));
            }
        });

        // Animation.
        ui.collapsing("Animation", |ui| {
            if ui
                .add(Slider::new(&mut state.animation_speed, 1..=10).text("Speed"))
                .changed()
            {
                dispatch(UserEvent::AnimationSpeedChanged(state.animation_speed));
            }
            ui.horizontal(|ui| {
                if ui.add(animation_button(state)).clicked() {
                    dispatch(UserEvent::ToggleAnimation);
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
    Slider::new(&mut state.df_resolution, 64..=MAX_DISTANCE_FIELD_RESOLUTION).text("SES resolution")
}

fn probe_radius_slider(state: &mut UIState) -> Slider {
    Slider::new(
        &mut state.probe_radius,
        DEFAULT_PROBE_RADIUS..=MAX_PROBE_RADIUS,
    )
    .text("Probe radius")
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

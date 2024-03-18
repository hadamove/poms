use egui::{Button, Checkbox, Pos2, Slider, Window};

use crate::app::{
    constants::{
        ANIMATION_ACTIVE_BY_DEFAULT, DEFAULT_LIGHT_COLOR, DEFAULT_LIGHT_DIRECTION,
        DEFAULT_PROBE_RADIUS, DEFAULT_SES_RESOLUTION, MAX_PROBE_RADIUS, MAX_SES_RESOLUTION,
    },
    dtos::LightData,
    ui::event::UserEvent,
};

pub struct SettingsState {
    ses_resolution: u32,
    probe_radius: f32,
    render_spacefill: bool,
    render_ses: bool,

    light_follow_camera: bool,
    light_direction: [f32; 3],
    light_color: [f32; 3],

    is_animation_active: bool,
    animation_speed: u32,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            ses_resolution: DEFAULT_SES_RESOLUTION,
            probe_radius: DEFAULT_PROBE_RADIUS,
            render_spacefill: true,
            render_ses: true,

            light_follow_camera: true,
            light_direction: DEFAULT_LIGHT_DIRECTION,
            light_color: DEFAULT_LIGHT_COLOR,

            is_animation_active: ANIMATION_ACTIVE_BY_DEFAULT,
            animation_speed: 5,
        }
    }
}

pub fn settings(
    context: &egui::Context,
    state: &mut SettingsState,
    dispatch: &mut dyn FnMut(UserEvent),
) {
    let window = Window::new("Settings")
        .default_pos(Pos2::new(100.0, 100.0))
        .default_width(100.0);

    window.show(context, |ui| {
        // Model parameters.
        if ui.add(resolution_slider(state)).changed() {
            dispatch(UserEvent::SesResolutionChanged(state.ses_resolution));
        }
        if ui.add(probe_radius_slider(state)).changed() {
            dispatch(UserEvent::ProbeRadiusChanged(state.probe_radius));
        }
        ui.separator();

        ui.collapsing("Render Passes", |ui| {
            if ui.add(spacefill_pass_checkbox(state)).changed() {
                dispatch(UserEvent::RenderSpacefillChanged(state.render_spacefill));
            }
            if ui.add(ses_pass_checkbox(state)).changed() {
                dispatch(UserEvent::RenderSesChanged(state.render_ses));
            }
        });

        // Light options.
        ui.collapsing("Lighting", |ui| {
            if ui
                .add(Checkbox::new(
                    &mut state.light_follow_camera,
                    "Light follows camera",
                ))
                .changed()
            {
                dispatch(UserEvent::UpdateLight(LightData {
                    follow_camera: Some(state.light_follow_camera),
                    ..Default::default()
                }));
            }

            if !state.light_follow_camera {
                ui.label("Light direction:");
                for (i, &coord) in ["X", "Y", "Z"].iter().enumerate() {
                    if ui
                        .add(Slider::new(&mut state.light_direction[i], -1.0..=1.0).text(coord))
                        .changed()
                    {
                        dispatch(UserEvent::UpdateLight(LightData {
                            direction: Some(state.light_direction),
                            ..Default::default()
                        }));
                    }
                }
            }

            ui.label("Light color:");
            if ui.color_edit_button_rgb(&mut state.light_color).changed() {
                dispatch(UserEvent::UpdateLight(LightData {
                    color: Some(state.light_color),
                    ..Default::default()
                }));
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

fn resolution_slider(state: &mut SettingsState) -> Slider {
    Slider::new(&mut state.ses_resolution, 64..=MAX_SES_RESOLUTION).text("SES resolution")
}

fn probe_radius_slider(state: &mut SettingsState) -> Slider {
    Slider::new(
        &mut state.probe_radius,
        DEFAULT_PROBE_RADIUS..=MAX_PROBE_RADIUS,
    )
    .text("Probe radius")
}

fn spacefill_pass_checkbox(state: &mut SettingsState) -> Checkbox {
    Checkbox::new(&mut state.render_spacefill, "Spacefill")
}

fn ses_pass_checkbox(state: &mut SettingsState) -> Checkbox {
    Checkbox::new(&mut state.render_ses, "SES")
}

fn animation_button(state: &mut SettingsState) -> Button {
    match state.is_animation_active {
        true => Button::new("⏸"),
        false => Button::new("▶"),
    }
}

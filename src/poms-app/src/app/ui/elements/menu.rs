use egui::{menu, TopBottomPanel, Visuals};

use crate::app::constants::ColorTheme;
use crate::app::ui::{events::UserEvent, state::UIState};

/// Component that displays the menu bar.
/// Contains buttons for opening files and changing visual theme.
pub fn menu_bar(context: &egui::Context, state: &mut UIState) {
    TopBottomPanel::top("menu_bar").show(context, |ui| {
        menu::bar(ui, |ui| {
            // Visual theme toggle
            if ui.visuals().dark_mode {
                if ui.button("🔆").clicked() {
                    context.set_visuals(Visuals::light());
                    state.dispatch_event(UserEvent::ToggleTheme {
                        theme: ColorTheme::Light,
                    });
                }
            } else if ui.button("🌙").clicked() {
                context.set_visuals(Visuals::dark());
                state.dispatch_event(UserEvent::ToggleTheme {
                    theme: ColorTheme::Dark,
                });
            }

            ui.menu_button("File", |ui| {
                if ui.button("Open").clicked() {
                    state.dispatch_event(UserEvent::OpenFileDialog);
                }
            });
        });
    });
}

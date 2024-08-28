use std::collections::HashSet;

use egui::{Align, Layout, Window};

use crate::app::ui::state::UIState;

/// Component that displays error messages and allows to close them.
pub fn error_messages(context: &mut egui::Context, state: &mut UIState) {
    let mut closed_errors = HashSet::new();
    for error in &state.error_messages {
        Window::new("Error")
            .collapsible(false)
            .default_pos((256.0, 256.0))
            .show(context, |ui| {
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    ui.label(&error.message);
                    if ui.button("Close").clicked() {
                        // Mark the error message for removal
                        closed_errors.insert(error.id);
                    }
                });
            });
    }

    state
        .error_messages
        .retain(|error| !closed_errors.contains(&error.id));
}

use egui::Window;

use crate::app::ui::{events::UserEvent, state::UIState};

pub fn file_menu(context: &egui::Context, state: &mut UIState) {
    let mut clicked_molecule: Option<usize> = None;

    Window::new("Files opened")
        .scroll([false, true])
        .default_size([100.0, 100.0])
        .default_pos(egui::Pos2::new(100.0, 200.0))
        .show(context, |ui| {
            for file in &state.files_loaded {
                // Highlight the active file
                let button = if file.index == state.active_file_index {
                    ui.button(&file.path).highlight()
                } else {
                    ui.button(&file.path)
                };

                if button.clicked() {
                    clicked_molecule = Some(file.index);
                }
            }
        });

    if let Some(index) = clicked_molecule {
        state.dispatch_event(UserEvent::ActivateFile { index });
    }
}

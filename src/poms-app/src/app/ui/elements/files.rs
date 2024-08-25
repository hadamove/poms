use egui::{RichText, Widget, Window};

use crate::app::ui::{events::UserEvent, state::UIState};

pub fn file_menu(context: &egui::Context, state: &mut UIState) {
    let mut clicked_molecule: Option<usize> = None;

    Window::new("Files")
        .default_size([200.0, 100.0])
        .anchor(egui::Align2::RIGHT_TOP, [-16.0, 36.0])
        .show(context, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for file in &state.files_loaded {
                    let mut text = RichText::new(&file.path).small();
                    if file.index == state.active_file_index {
                        text = text.strong();
                    }

                    let button = ui.add_sized(
                        [ui.available_width(), 16.0],
                        egui::Button::new(text)
                            .small()
                            .shortcut_text(".")
                            .frame(false),
                    );

                    if button.clicked() {
                        clicked_molecule = Some(file.index);
                    }

                    egui::Separator::default().spacing(3.0).ui(ui);
                }
            });

            if ui.button("Open...").clicked() {
                state.dispatch_event(UserEvent::OpenFileDialog);
            }
        });

    if let Some(index) = clicked_molecule {
        state.dispatch_event(UserEvent::ActivateFile { index });
    }
}

use egui::Widget;

use crate::app::ui::{events::UserEvent, state::UIState};

pub(crate) fn file_menu(context: &mut egui::Context, state: &mut UIState) {
    let mut clicked_molecule: Option<usize> = None;

    let top_right = [context.screen_rect().width() - 16.0, 36.0];

    egui::Window::new("Loaded Files")
        .default_size([256.0, 256.0])
        .max_height(512.0)
        .pivot(egui::Align2::RIGHT_TOP)
        .default_pos(top_right)
        .resizable(false)
        .show(context, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for file in state.files_loaded.iter() {
                    let prefix = if file.index == state.active_file_index {
                        "▶"
                    } else {
                        " "
                    };
                    let text = egui::RichText::new(format!("{}{}", prefix, &file.path)).small();

                    let button = ui.add_sized(
                        [ui.available_width(), 16.0],
                        egui::Button::new(text)
                            .small()
                            .shortcut_text(format!("{} atoms", file.number_of_atoms))
                            .frame(false),
                    );

                    if button.clicked() {
                        clicked_molecule = Some(file.index);
                    }

                    egui::Separator::default().spacing(3.0).ui(ui);
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Open...").clicked() {
                    state.dispatch_event(UserEvent::InitOpenFileDialog);
                }

                if ui.button("Search PDB...").clicked() {
                    state.is_search_window_shown = true;
                    state.is_search_first_time_rendered = true;
                }

                // Show the delete button only if there are more than one molecules loaded.
                ui.add_enabled_ui(state.files_loaded.len() > 1, |ui| {
                    if ui.button("🗑").clicked() {
                        state.dispatch_event(UserEvent::DeleteActiveMolecule);
                    }
                })
            });
        });

    if let Some(index) = clicked_molecule {
        state.dispatch_event(UserEvent::ChangeActiveMolecule { index });
    }
}

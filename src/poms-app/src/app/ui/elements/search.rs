use egui::{RichText, Widget, Window};

use crate::app::{
    data::pdb_apis::Assembly,
    ui::{events::UserEvent, state::UIState},
};

pub fn search(context: &egui::Context, state: &mut UIState) {
    let mut clicked_result: Option<Assembly> = None;

    if !state.is_search_window_shown {
        return;
    }

    Window::new("Search")
        .default_pos(context.screen_rect().center() + egui::Vec2::new(400., 50.))
        .default_size([250., 100.])
        .show(context, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                if egui::TextEdit::singleline(&mut state.search_term)
                    .hint_text("Search and download data from rcsb.org..")
                    .desired_width(f32::INFINITY)
                    .ui(ui)
                    .changed()
                {
                    state.is_search_in_progress = true;
                    state.dispatch_event(UserEvent::InitMoleculeSearch {
                        query: state.search_term.clone(),
                    })
                };

                egui::Separator::default().spacing(6.0).ui(ui);

                if state.is_search_in_progress {
                    ui.label("Searching...");
                    egui::Separator::default().spacing(3.0).ui(ui);
                } else if state.search_results.is_empty() && !state.search_term.is_empty() {
                    ui.label("No results found.");
                    egui::Separator::default().spacing(3.0).ui(ui);
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(250.)
                        .show(ui, |ui| {
                            for result in &state.search_results {
                                let text = RichText::new(result.to_string()).small();

                                let button = ui.add_sized(
                                    [ui.available_width(), 16.0],
                                    egui::Button::new(text)
                                        .small()
                                        .shortcut_text(".")
                                        .frame(false),
                                );

                                if button.clicked() {
                                    clicked_result = Some(result.clone());
                                }

                                egui::Separator::default().spacing(3.0).ui(ui);
                            }
                        });
                }

                if ui.button("Close").clicked() {
                    state.is_search_window_shown = false;
                }
            });
        });

    if let Some(assembly) = clicked_result {
        state.dispatch_event(UserEvent::InitDownloadMolecule { assembly });
    }
}

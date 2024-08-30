use egui::{RichText, Widget, Window};

use crate::app::{
    data::{file_loader::DownloadProgress, Assembly},
    ui::{events::UserEvent, state::UIState},
};

pub(crate) fn search(context: &mut egui::Context, state: &mut UIState) {
    let mut clicked_result: Option<Assembly> = None;

    if !state.is_search_window_shown {
        return;
    }

    Window::new("Search")
        .default_pos(context.screen_rect().center() + egui::Vec2::new(400., 50.))
        .default_size([250., 100.])
        .show(context, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                let search_bar = egui::TextEdit::singleline(&mut state.search_term)
                    .hint_text("Search and download data from rcsb.org..")
                    .desired_width(f32::INFINITY)
                    .ui(ui);

                if state.is_search_first_time_rendered {
                    context.memory_mut(|m| {
                        m.request_focus(search_bar.id);
                    });
                    state.is_search_first_time_rendered = false;
                }

                if search_bar.changed() && !state.search_term.is_empty() {
                    state.is_search_in_progress = true;
                    state.dispatch_event(UserEvent::InitMoleculeSearch {
                        query: state.search_term.clone(),
                    })
                };

                egui::Separator::default().spacing(6.0).ui(ui);

                if !state.search_term.is_empty() && state.is_search_in_progress {
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

                                if button.clicked() && state.download_progress.is_none() {
                                    clicked_result = Some(result.clone());
                                }

                                egui::Separator::default().spacing(3.0).ui(ui);
                            }
                        });
                }

                if let Some(DownloadProgress::Downloading { bytes_downloaded }) =
                    state.download_progress
                {
                    ui.label(format!(
                        "Downloading.. {:.2} MB",
                        bytes_downloaded as f64 / 1024. / 1024.
                    ));
                    egui::Separator::default().spacing(3.0).ui(ui);
                } else if let Some(DownloadProgress::Parsing) = state.download_progress {
                    ui.label("Downloaded.. Parsing..");
                    egui::Separator::default().spacing(3.0).ui(ui);
                }

                if ui.button("Close").clicked() {
                    state.is_search_window_shown = false;
                }
            });
        });

    if let Some(assembly) = clicked_result {
        state.download_progress = Some(DownloadProgress::Downloading {
            bytes_downloaded: 0,
        });
        state.dispatch_event(UserEvent::InitDownloadMolecule { assembly });
    }
}

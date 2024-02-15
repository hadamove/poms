use egui::{Align, Layout, Window};

pub struct ErrorMessageState {
    message: String,
    should_close: bool,
}

pub fn error_message(context: &egui::Context, error: &mut ErrorMessageState) {
    Window::new("Error")
        .collapsible(false)
        .default_pos((256.0, 256.0))
        .show(context, |ui| {
            ui.with_layout(Layout::top_down(Align::Center), |ui| {
                ui.label(&error.message);
                if ui.button("Close").clicked() {
                    error.should_close = true;
                }
            });
        });
}

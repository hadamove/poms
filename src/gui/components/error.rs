use egui::{Align, Layout, Window};

use super::super::GuiEvents;
use super::GuiComponent;

pub struct ErrorMessage {
    message: String,
    should_close: bool,
}

impl ErrorMessage {
    pub fn new(message: String) -> Self {
        Self {
            message,
            should_close: false,
        }
    }
}

impl GuiComponent for ErrorMessage {
    fn draw(&mut self, context: &egui::Context, _events: &mut GuiEvents) {
        // Clone so that we can mutate self.message.
        Window::new("Error")
            .collapsible(false)
            .default_pos((256.0, 256.0))
            .show(context, |ui| {
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    ui.label(&self.message);
                    if ui.button("Close").clicked() {
                        self.should_close = true;
                    }
                });
            });
    }

    fn should_close(&self) -> bool {
        self.should_close
    }
}

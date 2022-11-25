use egui::{Align, Layout, Window};

use super::GuiComponent;
use crate::shared::events::{AppEvent, EventDispatch};

#[derive(Default)]
pub struct ErrorMessage {
    message: Option<String>,
}

impl GuiComponent for ErrorMessage {
    fn draw(&mut self, context: &egui::Context, _dispatch: &EventDispatch) {
        if let Some(message) = &self.message {
            // Clone so that we can mutate self.message.
            let message_clone = message.clone();
            Window::new("Error")
                .collapsible(false)
                .default_pos((256.0, 256.0))
                .show(context, |ui| {
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        ui.label(message_clone);
                        if ui.button("Close").clicked() {
                            self.message = None;
                        }
                    });
                });
        }
    }

    fn update(&mut self, event: &AppEvent) {
        if let AppEvent::DisplayError(message) = event {
            self.message = Some(message.clone());
        }
    }
}

use super::GuiEvents;

pub(super) mod error;
pub(super) mod menu;
pub(super) mod settings;

pub(super) trait GuiComponent {
    fn draw(&mut self, context: &egui::Context, events: &mut GuiEvents);
    fn should_close(&self) -> bool;
}

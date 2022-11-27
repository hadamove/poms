use egui::{menu, TopBottomPanel};

use crate::gui::GuiEvent;

use super::super::GuiEvents;
use super::GuiComponent;

#[derive(Default)]
pub struct Menu;

impl GuiComponent for Menu {
    fn draw(&mut self, context: &egui::Context, events: &mut GuiEvents) {
        TopBottomPanel::top("menu_bar").show(context, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Load molecules").clicked() {
                        events.push(GuiEvent::OpenFileDialog);
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Organize windows").clicked() {
                        ui.ctx().memory().reset_areas();
                        ui.close_menu();
                    }
                });
            });
        });
    }

    fn should_close(&self) -> bool {
        false
    }
}

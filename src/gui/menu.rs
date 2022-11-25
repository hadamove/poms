use egui::{menu, TopBottomPanel};

use super::GuiComponent;
use crate::shared::events::{AppEvent, EventDispatch};

#[derive(Default)]
pub struct Menu;

impl GuiComponent for Menu {
    fn draw(&mut self, context: &egui::Context, dispatch: &EventDispatch) {
        TopBottomPanel::top("menu_bar").show(context, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Load molecules").clicked() {
                        let x = dispatch.send(AppEvent::OpenFileDialogRequested).ok();
                        println!("Load molecules sentttt {:?}", x);
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

    fn update(&mut self, _event: &AppEvent) {}
}

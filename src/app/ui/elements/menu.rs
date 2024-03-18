use egui::{menu, TopBottomPanel, Visuals};

use crate::app::{constants::ColorTheme, ui::event::UserEvent};

pub fn menu_bar(context: &egui::Context, dispatch: &mut dyn FnMut(UserEvent)) {
    TopBottomPanel::top("menu_bar").show(context, |ui| {
        menu::bar(ui, |ui| {
            // Visual theme toggle
            if ui.visuals().dark_mode {
                if ui.button("🔆").clicked() {
                    context.set_visuals(Visuals::light());
                    dispatch(UserEvent::ToggleTheme(ColorTheme::Light));
                }
            } else if ui.button("🌙").clicked() {
                context.set_visuals(Visuals::dark());
                dispatch(UserEvent::ToggleTheme(ColorTheme::Dark));
            }

            ui.menu_button("File", |ui| {
                if ui.button("Open").clicked() {
                    dispatch(UserEvent::OpenFileDialog);
                }
            });
        });
    });
}

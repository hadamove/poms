use error::ErrorMessage;
use menu::Menu;
use settings::UserSettings;

use crate::shared::events::{AppEvent, EventDispatch};

mod error;
mod menu;
mod settings;

pub struct Gui {
    components: Vec<Box<dyn GuiComponent>>,
    dispatch: EventDispatch,
}

impl Gui {
    pub fn new(dispatch: EventDispatch) -> Self {
        Self {
            components: vec![
                Box::new(Menu::default()),
                Box::new(UserSettings::default()),
                Box::new(ErrorMessage::default()),
            ],
            dispatch,
        }
    }

    pub fn ui(&mut self, context: &egui::Context) {
        for gui_component in self.components.iter_mut() {
            gui_component.ui(context, &self.dispatch);
        }
    }

    pub fn process_event(&mut self, event: &AppEvent) {
        for component in self.components.iter_mut() {
            component.update(event);
        }
    }
}

trait GuiComponent {
    fn ui(&mut self, context: &egui::Context, dispatch: &EventDispatch);
    fn update(&mut self, event: &AppEvent);
}

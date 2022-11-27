use egui::FullOutput;
use egui_winit_platform::{Platform, PlatformDescriptor};
use error::ErrorMessage;
use menu::Menu;
use settings::UserSettings;
use winit::event::Event;

use crate::{
    context::Context,
    shared::events::{AppEvent, EventDispatch},
};

mod error;
mod menu;
mod settings;

pub struct GuiOutput(pub FullOutput, pub egui::Context);

pub struct GuiEvents {
    pub events: Vec<AppEvent>,
}

pub struct Gui {
    components: Vec<Box<dyn GuiComponent>>,
    // Integration between egui and winit.
    platform: Platform,
    dispatch: EventDispatch,
}

impl Gui {
    pub fn new(context: &Context, dispatch: EventDispatch) -> Self {
        Self {
            components: vec![
                Box::new(Menu::default()),
                Box::new(UserSettings::default()),
                Box::new(ErrorMessage::default()),
            ],
            platform: Platform::new(PlatformDescriptor {
                physical_width: context.config.width,
                physical_height: context.config.height,
                scale_factor: context.scale_factor,
                ..Default::default()
            }),
            dispatch,
        }
    }

    pub fn draw_frame(&mut self) -> GuiOutput {
        let context = self.platform.context();
        self.platform.begin_frame();

        for gui_component in self.components.iter_mut() {
            gui_component.draw(&context, &self.dispatch);
        }
        let output = self.platform.end_frame(None);

        // Return the shapes and text to be drawn by render pass.
        GuiOutput(output, context)
    }

    pub fn handle_app_event(&mut self, event: &AppEvent) {
        for component in self.components.iter_mut() {
            component.update(event);
        }
    }

    pub fn handle_winit_event<T>(&mut self, winit_event: &Event<T>) -> bool {
        self.platform.handle_event(winit_event);
        self.platform.captures_event(winit_event)
    }
}

trait GuiComponent {
    fn draw(&mut self, context: &egui::Context, dispatch: &EventDispatch);
    fn update(&mut self, event: &AppEvent);
}

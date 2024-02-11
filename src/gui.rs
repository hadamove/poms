use egui_winit_platform::{Platform, PlatformDescriptor};
use winit::event::Event;

use crate::{
    context::Context,
    parser::parse::ParsedFile,
    utils::{constants::ColorTheme, dtos::LightData},
};

use async_file::{AsyncFileLoader, FileResponse};
use components::{error::ErrorMessage, menu::Menu, settings::UserSettings, GuiComponent};

mod async_file;
mod components;

pub enum GuiEvent {
    OpenFileDialog,
    LoadedMolecules(Vec<ParsedFile>),
    SesResolutionChanged(u32),
    ProbeRadiusChanged(f32),
    RenderSpacefillChanged(bool),
    RenderSesChanged(bool),
    ToggleAnimation,
    ToggleTheme(ColorTheme),
    UpdateLight(LightData),
    AnimationSpeedChanged(u32),
}

pub type GuiEvents = Vec<GuiEvent>;

pub struct GuiOutput(pub egui::FullOutput, pub egui::Context);

pub struct Gui {
    components: Vec<Box<dyn GuiComponent>>,
    // Integration between egui and winit.
    platform: Platform,

    async_file: AsyncFileLoader,
}

impl Gui {
    pub fn new(context: &Context) -> Self {
        Self {
            components: vec![Box::<Menu>::default(), Box::<UserSettings>::default()],

            async_file: AsyncFileLoader::new(),
            platform: Platform::new(PlatformDescriptor {
                physical_width: context.config.width,
                physical_height: context.config.height,
                scale_factor: context.scale_factor,
                ..Default::default()
            }),
        }
    }

    pub fn draw_frame(&mut self) -> (GuiOutput, GuiEvents) {
        let mut events = GuiEvents::new();

        let context = self.platform.context();
        self.platform.begin_frame();

        self.draw_components(&context, &mut events);
        self.handle_internal_events(&mut events);

        let output = self.platform.end_frame(None);

        // Return the shapes and text to be drawn by render pass.
        (GuiOutput(output, context), events)
    }

    pub fn handle_winit_event<T: 'static>(&mut self, winit_event: &Event<T>) -> bool {
        self.platform.handle_event(winit_event);
        self.platform.captures_event(winit_event)
    }

    pub fn handle_internal_events(&mut self, events: &mut GuiEvents) {
        #[allow(clippy::single_match)]
        for event in events.iter() {
            match event {
                GuiEvent::OpenFileDialog => self.async_file.load_pdb_files(),
                _ => {}
            }
        }
        self.handle_new_files(events)
    }

    fn draw_components(&mut self, context: &egui::Context, events: &mut GuiEvents) {
        for component in self.components.iter_mut() {
            component.draw(context, events);
        }
        // Remove components that should be closed.
        self.components.retain(|c| !c.should_close());
    }

    fn handle_new_files(&mut self, events: &mut GuiEvents) {
        match self.async_file.get_parsed_files() {
            FileResponse::ParsedFiles(files) => events.push(GuiEvent::LoadedMolecules(files)),
            FileResponse::ParsingFailed(err) => self.spawn_error(err.to_string()),
            FileResponse::NoContent => {}
        }
    }

    fn spawn_error(&mut self, message: String) {
        self.components.push(Box::new(ErrorMessage::new(message)));
    }

    #[cfg(target_arch = "wasm32")]
    // Hot-fix for GUI not resizing with the window in the browser. There is probably a better way to fix this.
    pub fn force_resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, context: &Context) {
        let raw_input = self.platform.raw_input_mut();
        raw_input.screen_rect = Some(egui::Rect::from_min_size(
            Default::default(),
            egui::vec2(new_size.width as f32, new_size.height as f32) / context.scale_factor as f32,
        ));
    }
}

use std::sync::Arc;

use egui::{ClippedPrimitive, FullOutput};
use egui_winit::EventResponse;

use winit::{event::WindowEvent, window::Window};

use crate::{
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

pub struct GuiOutput(pub egui::TexturesDelta, pub Vec<ClippedPrimitive>);

pub struct Gui {
    window: Arc<Window>,
    state: egui_winit::State,
    context: egui::Context,

    // TODO: Please refactor this
    components: Vec<Box<dyn GuiComponent>>,
    async_file: AsyncFileLoader,
}

impl Gui {
    pub fn new(window: Arc<Window>) -> Self {
        let context = egui::Context::default();

        let state = egui_winit::State::new(
            context.clone(),
            egui::ViewportId::ROOT,
            window.as_ref(),
            Some(window.scale_factor() as f32),
            None,
        );

        Self {
            window,
            context,
            state,
            components: vec![Box::<Menu>::default(), Box::<UserSettings>::default()],
            async_file: AsyncFileLoader::new(),
        }
    }

    pub fn process_frame(&mut self) -> (GuiOutput, GuiEvents) {
        let mut events = GuiEvents::new();

        let egui_input = self.state.take_egui_input(&self.window);
        self.context.begin_frame(egui_input);

        self.draw_components(&mut events);
        self.handle_internal_events(&mut events);

        let FullOutput {
            shapes,
            textures_delta,
            ..
        } = self.context.end_frame();

        let paint_jobs = self.context.tessellate(shapes, 1.0);

        // Return the shapes and text to be drawn by render pass.
        (GuiOutput(textures_delta, paint_jobs), events)
    }

    pub fn handle_winit_event(&mut self, window_event: &WindowEvent) -> bool {
        let EventResponse { consumed, .. } = self.state.on_window_event(&self.window, window_event);

        consumed
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

    fn draw_components(&mut self, events: &mut GuiEvents) {
        for component in self.components.iter_mut() {
            component.draw(&self.context, events);
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

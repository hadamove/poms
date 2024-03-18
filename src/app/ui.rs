use winit::event::WindowEvent;

use self::event::UserEvent;
use super::context::Context;
use super::file::{AsyncFileLoader, FileResponse};
use super::input::Input;

pub mod egui_wrapper;
pub mod elements;
pub mod event;

pub struct UserInterface {
    egui: egui_wrapper::EguiWrapper,

    settings: elements::SettingsState,
    active_errors: Vec<elements::ErrorMessageState>,

    file_loader: AsyncFileLoader,

    // TODO: make it private and unite with events
    pub input: Input,
}

impl UserInterface {
    pub fn new(context: &Context) -> Self {
        let egui = egui_wrapper::EguiWrapper::new(context);

        Self {
            egui,
            settings: elements::SettingsState::default(),
            active_errors: Vec::new(),
            file_loader: AsyncFileLoader::new(),

            input: Input::default(),
        }
    }

    pub fn process_frame(&mut self) -> Vec<UserEvent> {
        self.egui.begin_frame();

        let context = &self.egui.egui_handle;
        let mut events = Vec::<UserEvent>::new();
        let mut dispatch = |event: UserEvent| {
            events.push(event);
        };

        elements::menu_bar(context, &mut dispatch);
        elements::settings(context, &mut self.settings, &mut dispatch);
        for error in &mut self.active_errors {
            elements::error_message(context, error);
        }

        self.egui.end_frame();

        // TODO: Get rid of this
        self.handle_internal_events(&mut events);

        events
    }

    pub fn render(
        &mut self,
        context: &Context,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        self.egui.render(context, view, encoder);
        // TODO: Move this elsewhere
        self.input.reset();
    }

    pub fn handle_winit_event(&mut self, window_event: &WindowEvent) -> bool {
        self.egui.handle_winit_event(window_event)
    }

    // TODO: Solve this stuff
    pub fn handle_internal_events(&mut self, events: &mut Vec<UserEvent>) {
        #[allow(clippy::single_match)]
        for event in events.iter() {
            match event {
                UserEvent::OpenFileDialog => self.file_loader.load_pdb_files(),
                _ => {}
            }
        }
        self.handle_new_files(events)
    }

    fn handle_new_files(&mut self, events: &mut Vec<UserEvent>) {
        match self.file_loader.get_parsed_files() {
            FileResponse::FileParsed(file) => events.push(UserEvent::LoadedMolecule(file)),
            FileResponse::ParsingFailed(err) => {
                eprintln!("Parsing failed: {}", err);
                // TODO: Show error message in UI
            }
            FileResponse::NoContent => {}
        }
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

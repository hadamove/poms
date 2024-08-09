use state::UIState;
use winit::event::WindowEvent;

use self::event::UserEvent;
use super::context::Context;
use super::file::{AsyncFileLoader, FileResponse};
use super::input::Input;

pub mod egui_wrapper;
pub mod elements;
pub mod event;
mod state;

/// TODO: docs
pub struct UserInterface {
    egui: egui_wrapper::EguiWrapper,
    state: UIState,
    file_loader: AsyncFileLoader,

    // TODO: make it private and unite with events
    pub input: Input,
}

impl UserInterface {
    pub fn new(context: &Context) -> Self {
        let egui = egui_wrapper::EguiWrapper::new(context);

        Self {
            egui,
            state: UIState::default(),
            file_loader: AsyncFileLoader::new(),
            input: Input::default(),
        }
    }

    pub fn process_frame(&mut self) -> Vec<UserEvent> {
        self.egui.add_elements(
            &mut self.state,
            &[
                elements::menu_bar,
                elements::settings,
                elements::error_messages,
            ],
        );

        self.process_file_loader_events();

        self.state.collect_events()
    }

    pub fn render(
        &mut self,
        context: &Context,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        self.egui.render(context, view, encoder);
    }

    pub fn handle_window_event(&mut self, window_event: &WindowEvent) -> bool {
        self.egui.handle_window_event(window_event) || self.input.handle_window_event(window_event)
    }

    pub fn open_file_dialog(&mut self) {
        self.file_loader.open_file_dialog();
    }

    fn process_file_loader_events(&mut self) {
        match self.file_loader.get_parsed_files() {
            FileResponse::FileParsed { molecule } => self
                .state
                .dispatch_event(UserEvent::LoadedMolecule { molecule }),
            FileResponse::ParsingFailed { error } => self
                .state
                .open_error_message(format!("Parsing failed: {}", error)),
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

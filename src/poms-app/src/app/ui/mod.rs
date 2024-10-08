mod elements;
pub(crate) mod events;
mod glue;
pub(crate) mod state;

use poms_compute::ComputeProgress;
use winit::event::WindowEvent;

use super::data::file_loader::{AsyncWorkResult, DownloadProgress, FileLoader};
use super::data::molecule_storage::MoleculeData;
use crate::gpu_context::GpuContext;
use events::UserEvent;
use state::{MoleculeFileInfo, UIState};

/// Primary struct for managing and rendering the application's UI and I/O.
pub(crate) struct UserInterface {
    /// An abstraction for loading files from the user's filesystem.
    pub(crate) file_loader: FileLoader,
    /// Holds the state of interactive elements (e.g. buttons, sliders) and user interactions.
    state: UIState,
    /// A thin wrapper around the `egui` library, providing an abstraction for the UI system.
    egui_wrapper: glue::EguiWrapper,
}

impl UserInterface {
    /// Creates a new `UserInterface` using the provided `GpuContext`.
    pub(crate) fn new(context: &GpuContext, initial_state: UIState) -> Self {
        let egui_wrapper = glue::EguiWrapper::new(context);

        Self {
            egui_wrapper,
            state: initial_state,
            file_loader: FileLoader::new(),
        }
    }

    /// Processes a single UI frame, updating all registered UI elements and handling file loader events.
    /// The method also gathers and returns a list of events that were generated during the frame,
    /// which can be used to trigger further application logic.
    pub(crate) fn process_frame(&mut self) -> Vec<UserEvent> {
        self.egui_wrapper.add_elements(
            &mut self.state,
            &[
                elements::menu_bar,
                elements::settings,
                elements::error_messages,
                elements::file_menu,
                elements::search,
            ],
        );

        self.process_file_loader_events();

        self.state.drain_events()
    }

    /// Renders the current UI frame to the specified texture view.
    /// Call this method after processing the frame using `process_frame`.
    pub(crate) fn render(
        &mut self,
        context: &GpuContext,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        self.egui_wrapper.render(context, view, encoder);
    }

    /// Handles a window event, routing them to the `egui` wrapper. Returns `true` if the event was consumed.
    pub(crate) fn handle_window_event(&mut self, window_event: &WindowEvent) -> bool {
        self.egui_wrapper.handle_window_event(window_event)
    }

    /// Updates the state of loaded files and the active file index.
    pub(crate) fn update_files_state(
        &mut self,
        molecule_files: &[MoleculeData],
        active_index: usize,
    ) {
        self.state.files_loaded = molecule_files
            .iter()
            .enumerate()
            .map(|(i, file)| MoleculeFileInfo {
                index: i,
                path: file.filename.clone(),
                number_of_atoms: file.atoms.data.len(),
            })
            .collect();

        self.state.active_file_index = active_index;
    }

    /// Updates the displayed progress of the compute pipeline.
    pub(crate) fn update_compute_progress(&mut self, progress: Option<ComputeProgress>) {
        self.state.compute_progress = progress;
    }

    fn process_file_loader_events(&mut self) {
        for event in self.file_loader.collect_data_events() {
            match event {
                AsyncWorkResult::FilesParsed { result: files } => match files {
                    Ok(files) => self
                        .state
                        .dispatch_event(UserEvent::MoleculesParsed { molecules: files }),
                    Err(error) => self
                        .state
                        .open_error_message(format!("Parsing failed: {}", error)),
                },
                AsyncWorkResult::SearchResultsParsed { result: results } => match results {
                    Ok(search_results) => {
                        self.state.search_results = search_results;
                        self.state.is_search_in_progress = false;
                    }
                    Err(error) => {
                        self.state.search_results = vec![];
                        eprintln!("Search failed: {}", error);
                    }
                },
                AsyncWorkResult::DownloadProgressed { progress } => {
                    self.state.download_progress = match progress {
                        // Reset progress if it's already finished
                        DownloadProgress::Finished => None,
                        _ => Some(progress),
                    };
                }
            }
        }
    }
}

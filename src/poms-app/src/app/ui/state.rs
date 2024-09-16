use super::events::UserEvent;
use crate::app::data::{file_loader::DownloadProgress, Assembly};

/// Struct that represents an error message that should be displayed to the user.
pub(crate) struct ErrorMessage {
    pub(crate) id: uuid::Uuid,
    pub(crate) message: String,
}

/// Struct holding metadata about a loaded molecule file.
pub(crate) struct MoleculeFileInfo {
    /// Index within the `MoleculeStorage::loaded_molecules` vector.
    pub(crate) index: usize,
    /// Full path within the OS.
    pub(crate) path: String,
    /// N.o. of atoms in the molecule parsed from the file.
    pub(crate) number_of_atoms: usize,
}

/// Struct that holds current state of the UI.
/// Also used to store dispatched events that are collected by the main app loop.
#[derive(Default)]
pub(crate) struct UIState {
    /// Resolution of the distance field used for molecular surface rendering.
    pub(crate) target_resolution: u32,
    /// Probe radius used for molecular surface rendering.
    pub(crate) probe_radius: f32,
    /// Flag that indicates if spacefill pass should be rendered.
    pub(crate) render_spacefill: bool,
    /// Flag that indicates if molecular surface pass should be rendered.
    pub(crate) render_molecular_surface: bool,
    /// Settings for postprocessing effects.
    pub(crate) postprocess_settings: poms_render::PostprocessSettings,

    /// Flag that indicates if animation is active.
    pub(crate) is_animation_active: bool,
    /// Speed of the animation.
    pub(crate) animation_speed: u32,
    /// List of error messages that should be displayed.
    pub(crate) error_messages: Vec<ErrorMessage>,

    /// Index within the `MoleculeStorage::loaded_moleculse` vector, points to currently active file (one that is being rendered).
    pub(crate) active_file_index: usize,
    /// A vector holding metadata about all of the molecule files.
    pub(crate) files_loaded: Vec<MoleculeFileInfo>,

    /// State of the input field in the search bar.
    pub(crate) search_term: String,
    /// List of displayed search results. Obtained from the PDB file search API.
    pub(crate) search_results: Vec<Assembly>,
    /// Flag that indicates if the search window is shown.
    pub(crate) is_search_window_shown: bool,
    /// Flag that indicates if a search is currently in progress.
    pub(crate) is_search_in_progress: bool,
    /// Flag used for forcing focus to the search bar.
    pub(crate) is_search_first_time_rendered: bool,

    /// Keeps track of the download status if there is one in progress.
    pub(crate) download_progress: Option<DownloadProgress>,

    /// Keeps track of the progress of the compute process.
    pub(crate) compute_progress: Option<poms_compute::ComputeProgress>,

    /// List of events that were dispatched by the UI.
    pub(crate) events: Vec<UserEvent>,
}

impl UIState {
    /// Dispatches an event to the UI.
    pub(crate) fn dispatch_event(&mut self, event: UserEvent) {
        self.events.push(event);
    }

    /// Collects dispatched events and clears the list of events.
    /// Call this method at the end of the frame to collect all events that were dispatched during the frame.
    pub(crate) fn drain_events(&mut self) -> Vec<UserEvent> {
        self.events.drain(..).collect()
    }

    /// Adds a new error message to the list of error messages.
    /// Call this method to display an error message to the user.
    pub(crate) fn open_error_message(&mut self, message: String) {
        self.error_messages.push(ErrorMessage {
            id: uuid::Uuid::new_v4(),
            message,
        });
    }
}

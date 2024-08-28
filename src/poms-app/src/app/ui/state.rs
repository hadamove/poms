use crate::app::data::pdb_apis::Assembly;

use super::events::UserEvent;

/// Struct that represents an error message that should be displayed to the user.
pub struct ErrorMessage {
    pub id: uuid::Uuid,
    pub message: String,
}

/// Struct holding metadata about a loaded molecule file.
pub struct MoleculeFileInfo {
    /// Index within the `MoleculeStorage::loaded_molecules` vector.
    pub index: usize,
    /// Full path within the OS.
    pub path: String,
}

/// Struct that holds current state of the UI.
/// Also used to store dispatched events that are collected by the main app loop.
#[derive(Default)]
pub struct UIState {
    /// Resolution of the distance field used for molecular surface rendering.
    pub target_resolution: u32,
    /// Probe radius used for molecular surface rendering.
    pub probe_radius: f32,
    /// Flag that indicates if spacefill pass should be rendered.
    pub render_spacefill: bool,
    /// Flag that indicates if molecular surface pass should be rendered.
    pub render_molecular_surface: bool,
    /// Flag that indicates if animation is active.
    pub is_animation_active: bool,
    /// Speed of the animation.
    pub animation_speed: u32,
    /// List of error messages that should be displayed.
    pub error_messages: Vec<ErrorMessage>,

    /// Index within the `MoleculeStorage::loaded_moleculse` vector, points to currently active file (one that is being rendered).
    pub active_file_index: usize,
    /// A vector holding metadata about all of the molecule files.
    pub files_loaded: Vec<MoleculeFileInfo>,

    /// State of the input field in the search bar.
    pub search_term: String,
    /// List of displayed search results. Obtained from the PDB file search API.
    pub search_results: Vec<Assembly>,
    /// Flag that indicates if the search window is shown.
    pub is_search_window_shown: bool,
    /// Flag that indicates if a search is currently in progress.
    pub is_search_in_progress: bool,

    /// How many bytes have been downloaded so far.
    pub bytes_downloaded: usize,
    /// Flag that indicates if a download is currently in progress.
    pub is_download_in_progress: bool,

    /// List of events that were dispatched by the UI.
    pub events: Vec<UserEvent>,
}

impl UIState {
    /// Dispatches an event to the UI.
    pub fn dispatch_event(&mut self, event: UserEvent) {
        self.events.push(event);
    }

    /// Collects dispatched events and clears the list of events.
    /// Call this method at the end of the frame to collect all events that were dispatched during the frame.
    pub fn drain_events(&mut self) -> Vec<UserEvent> {
        self.events.drain(..).collect()
    }

    /// Adds a new error message to the list of error messages.
    /// Call this method to display an error message to the user.
    pub fn open_error_message(&mut self, message: String) {
        self.error_messages.push(ErrorMessage {
            id: uuid::Uuid::new_v4(),
            message,
        });
    }
}

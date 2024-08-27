use crate::app::data::molecule_parser::ParsedMolecule;
use crate::app::data::pdb_apis::Assembly;
use crate::app::theme::ColorTheme;

/// Represents an event that is triggered by the user interacting with the UI.
pub enum UserEvent {
    /// User clicks on a file in the file menu.
    ChangeActiveMolecule { index: usize },

    /// User changes the speed of the animation using a slider.
    ChangeAnimationSpeed { speed: u32 },

    /// User changes the resolution of the distance field used for molecular surface rendering using a slider.
    ChangeDistanceFieldResolution { resolution: u32 },

    /// User changes the radius of the probe used for molecular surface rendering.
    ChangeProbeRadius { probe_radius: f32 },

    /// User changes the visibility of the molecular surface pass.
    ChangeRenderMolecularSurface { is_enabled: bool },

    /// User changes the visibility of the spacefill pass.
    ChangeRenderSpacefill { is_enabled: bool },

    /// User clicks the "Open file" button in the UI. Opens a file dialog.
    InitOpenFileDialog,

    /// User changes the input field in the search bar and initiates a search for PDB files.
    InitMoleculeSearch { query: String },

    /// User clicks on a molecule in the search results. Initiates the download of the selected PDB file.
    InitDownloadMolecule { assembly: Assembly },

    /// User toggles the animation of multiple molecule frames.
    ToggleAnimation,

    /// User toggles the color theme of the application.
    ToggleTheme { theme: ColorTheme },

    /// The files with molecules were successfully loaded and parsed.
    MoleculesParsed { molecules: Vec<ParsedMolecule> },
}

pub enum AsyncEvent {}

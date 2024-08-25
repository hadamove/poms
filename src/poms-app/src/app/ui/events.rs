use crate::app::data::molecule_parser::ParsedMolecule;
use crate::app::theme::ColorTheme;

/// Represents an event that is triggered by the user interacting with the UI.
pub enum UserEvent {
    /// User clicks on a file in the file menu.
    ActivateFile { index: usize },

    /// User changes the speed of the animation using a slider.
    AnimationSpeedChanged { speed: u32 },

    /// User changes the resolution of the distance field used for molecular surface rendering using a slider.
    DistanceFieldResolutionChanged { resolution: u32 },

    /// The files with molecules were successfully loaded and parsed.
    MoleculesLoaded { molecules: Vec<ParsedMolecule> },

    /// User clicks the "Open file" button in the UI. Opens a file dialog.
    OpenFileDialog,

    /// User changes the radius of the probe used for molecular surface rendering.
    ProbeRadiusChanged { probe_radius: f32 },

    /// User changes the visibility of the molecular surface pass.
    RenderMolecularSurfaceChanged { is_enabled: bool },

    /// User changes the visibility of the spacefill pass.
    RenderSpacefillChanged { is_enabled: bool },

    /// User toggles the animation of multiple molecule frames.
    ToggleAnimation,

    /// User toggles the color theme of the application.
    ToggleTheme { theme: ColorTheme },
}

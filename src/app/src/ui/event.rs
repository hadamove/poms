use crate::{constants::ColorTheme, utils::parser::ParsedMolecule};

pub enum UserEvent {
    AnimationSpeedChanged { speed: u32 },
    DistanceFieldResolutionChanged { resolution: u32 },
    LoadedMolecule { molecule: ParsedMolecule },
    OpenErrorMessage { error: String },
    OpenFileDialog,
    ProbeRadiusChanged { probe_radius: f32 },
    RenderMolecularSurfaceChanged { is_enabled: bool },
    RenderSpacefillChanged { is_enabled: bool },
    ToggleAnimation,
    ToggleTheme { theme: ColorTheme },
    UpdateLight { direction: cgmath::Vector3<f32> },
}

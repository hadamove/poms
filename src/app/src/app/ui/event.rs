use crate::app::{constants::ColorTheme, data::molecule_parser::ParsedMolecule};

pub enum UserEvent {
    AnimationSpeedChanged { speed: u32 },
    DistanceFieldResolutionChanged { resolution: u32 },
    LoadedMolecule { molecule: ParsedMolecule },
    OpenFileDialog,
    ProbeRadiusChanged { probe_radius: f32 },
    RenderMolecularSurfaceChanged { is_enabled: bool },
    RenderSpacefillChanged { is_enabled: bool },
    ToggleAnimation,
    ToggleTheme { theme: ColorTheme },
}

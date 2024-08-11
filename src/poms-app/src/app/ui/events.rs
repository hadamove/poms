use crate::app::{data::molecule_parser::ParsedMolecule, theme::ColorTheme};

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

use crate::app::{constants::ColorTheme, dtos::LightData, utils::parser::ParsedMolecule};

pub enum UserEvent {
    OpenFileDialog,
    LoadedMolecule(ParsedMolecule),
    DistanceFieldResolutionChanged(u32),
    ProbeRadiusChanged(f32),
    RenderSpacefillChanged(bool),
    RenderMolecularSurfaceChanged(bool),
    ToggleAnimation,
    ToggleTheme(ColorTheme),
    UpdateLight(LightData),
    AnimationSpeedChanged(u32),
}

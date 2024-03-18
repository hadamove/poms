use crate::app::{constants::ColorTheme, dtos::LightData, utils::parser::ParsedMolecule};

pub enum UserEvent {
    OpenFileDialog,
    LoadedMolecule(ParsedMolecule),
    SesResolutionChanged(u32),
    ProbeRadiusChanged(f32),
    RenderSpacefillChanged(bool),
    RenderSesChanged(bool),
    ToggleAnimation,
    ToggleTheme(ColorTheme),
    UpdateLight(LightData),
    AnimationSpeedChanged(u32),
}

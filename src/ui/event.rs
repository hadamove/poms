use crate::parser::ParsedMolecule;
use crate::utils::{constants::ColorTheme, dtos::LightData};

pub enum UserEvent {
    OpenFileDialog,
    LoadedMolecules(Vec<ParsedMolecule>),
    SesResolutionChanged(u32),
    ProbeRadiusChanged(f32),
    RenderSpacefillChanged(bool),
    RenderSesChanged(bool),
    ToggleAnimation,
    ToggleTheme(ColorTheme),
    UpdateLight(LightData),
    AnimationSpeedChanged(u32),
}

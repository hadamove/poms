use crate::{
    app::{constants::ColorTheme, utils::parser::ParsedMolecule},
    render::resources::light::LightUniform,
};

// TODO: use structs e.g. AnimationSpeedChanged { speed: u32 }, DistanceFieldResolutionChanged { resolution: u32 }
pub enum UserEvent {
    AnimationSpeedChanged(u32),
    DistanceFieldResolutionChanged(u32),
    LoadedMolecule(ParsedMolecule),
    OpenErrorMessage(String),
    CloseErrorMessage(uuid::Uuid),
    OpenFileDialog,
    ProbeRadiusChanged(f32),
    RenderMolecularSurfaceChanged(bool),
    RenderSpacefillChanged(bool),
    ToggleAnimation,
    ToggleTheme(ColorTheme),
    UpdateLight(LightUniform),
}

use crate::{
    app::{constants::ColorTheme, utils::parser::ParsedMolecule},
    render::resources::light::LightUniform,
};

pub enum UserEvent {
    OpenFileDialog,
    LoadedMolecule(ParsedMolecule),
    DistanceFieldResolutionChanged(u32),
    ProbeRadiusChanged(f32),
    RenderSpacefillChanged(bool),
    RenderMolecularSurfaceChanged(bool),
    ToggleAnimation,
    ToggleTheme(ColorTheme),
    UpdateLight(LightUniform),
    AnimationSpeedChanged(u32),
}

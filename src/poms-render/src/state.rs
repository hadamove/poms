use super::RenderParameters;

/// Configuration for the renderer.
pub struct RenderSettings {
    /// Whether to render the spacefill representation.
    pub render_spacefill: bool,
    /// Whether to render the molecular surface representation.
    pub render_molecular_surface: bool,
}

impl<'a> From<&RenderParameters<'a>> for RenderSettings {
    fn from(params: &RenderParameters) -> Self {
        RenderSettings {
            render_spacefill: params.render_spacefill,
            render_molecular_surface: params.render_molecular_surface,
        }
    }
}

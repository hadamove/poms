use crate::RenderParameters;

/// Configuration for the renderer.
pub struct RenderState {
    /// Whether to render the spacefill representation.
    pub render_spacefill: bool,
    /// Whether to render the molecular surface representation.
    pub render_molecular_surface: bool,
    /// The clear color of the renderer.
    pub clear_color: wgpu::Color,
}

impl<'a> From<&RenderParameters<'a>> for RenderState {
    fn from(params: &RenderParameters) -> Self {
        RenderState {
            render_spacefill: params.render_spacefill,
            render_molecular_surface: params.render_molecular_surface,
            clear_color: params.clear_color,
        }
    }
}

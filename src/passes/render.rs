use crate::context::Context;
use molecular_surface::RenderMolecularSurfacePass;
use spacefill::RenderSpacefillPass;

use self::molecular_surface::RenderMolecularSurfaceResources;
use self::spacefill::RenderSpacefillResources;

use super::resources::ResourceRepo;

mod molecular_surface;
mod spacefill;

mod util;

/// Configuration for the renderer.
pub struct RendererConfig {
    /// Whether to render the spacefill representation.
    pub render_spacefill: bool,
    /// Whether to render the molecular surface representation.
    pub render_molecular_surface: bool,
    /// The clear color of the renderer.
    pub clear_color: wgpu::Color,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            render_spacefill: false,
            render_molecular_surface: true,
            clear_color: wgpu::Color::BLACK,
        }
    }
}

/// A collection of render passes that are executed in order to render the molecule.
pub struct RenderJobs {
    spacefill_pass: RenderSpacefillPass,
    molecular_surface_pass: RenderMolecularSurfacePass,

    /// Configuration for the renderer. This is used to control what is rendered.
    pub config: RendererConfig,
}

impl RenderJobs {
    pub fn new(context: &Context, resources: &ResourceRepo) -> RenderJobs {
        let spacefill_pass = RenderSpacefillPass::new(
            &context.device,
            &context.config,
            RenderSpacefillResources {
                camera: resources.camera_resource.clone(),
                molecule: resources.molecule_resource.clone(),
            },
        );

        let molecular_surface_pass = RenderMolecularSurfacePass::new(
            &context.device,
            &context.config,
            RenderMolecularSurfaceResources {
                ses_grid: resources.ses_resource.clone(),
                df_texture: resources.df_texture_front.render.clone(),
                camera: resources.camera_resource.clone(),
                light: resources.light_resource.clone(),
            },
        );

        Self {
            spacefill_pass,
            molecular_surface_pass,
            config: RendererConfig::default(),
        }
    }

    pub fn execute(
        &mut self,
        _context: &Context,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        if self.config.render_spacefill {
            self.spacefill_pass
                .render(view, depth_view, encoder, self.config.clear_color);
        }

        if self.config.render_molecular_surface {
            self.molecular_surface_pass
                .render(view, depth_view, encoder, self.config.clear_color);
        }
    }
}

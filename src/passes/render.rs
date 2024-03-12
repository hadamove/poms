use molecular_surface::RenderMolecularSurfacePass;
use spacefill::RenderSpacefillPass;

use self::molecular_surface::RenderMolecularSurfaceResources;
use self::spacefill::RenderSpacefillResources;

use super::resources::camera::resource::CameraResource;
use super::resources::light::LightResource;
use super::resources::textures::depth_texture::DepthTexture;

use super::resources::CommonResources;

mod molecular_surface;
mod spacefill;

mod util;

pub struct RenderResources {
    pub depth_texture: DepthTexture,
    pub light_resource: LightResource,
    pub camera_resource: CameraResource,
    // TODO: Add df_texture
}

/// Configuration for the renderer.
pub struct RendererConfig {
    /// Whether to render the spacefill representation.
    pub render_spacefill: bool,
    /// Whether to render the molecular surface representation.
    pub render_molecular_surface: bool,
    /// The clear color of the renderer.
    pub clear_color: wgpu::Color,
    // TODO: Add ArcballCamera?
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
    /// Configuration for the renderer. This is used to control what is rendered.
    pub config: RendererConfig,

    /// The resources required for rendering. TODO: Better docs.
    pub resources: RenderResources,

    spacefill_pass: RenderSpacefillPass,
    molecular_surface_pass: RenderMolecularSurfacePass,
}

impl RenderJobs {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        common_resources: &CommonResources,
    ) -> RenderJobs {
        let resources = RenderResources {
            light_resource: LightResource::new(device),
            camera_resource: CameraResource::new(device),
            depth_texture: DepthTexture::new(device, config),
        };
        let spacefill_pass = RenderSpacefillPass::new(
            device,
            config,
            RenderSpacefillResources {
                camera: resources.camera_resource.clone(),
                molecule: common_resources.molecule_resource.clone(),
            },
        );

        let molecular_surface_pass = RenderMolecularSurfacePass::new(
            device,
            config,
            RenderMolecularSurfaceResources {
                ses_grid: common_resources.ses_resource.clone(),
                df_texture: common_resources.df_texture_front.render.clone(),
                camera: resources.camera_resource.clone(),
                light: resources.light_resource.clone(),
            },
        );

        Self {
            resources,
            spacefill_pass,
            molecular_surface_pass,
            config: RendererConfig::default(),
        }
    }

    pub fn execute(&mut self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        let depth_view = &self.resources.depth_texture.view;
        if self.config.render_spacefill {
            self.spacefill_pass
                .render(view, depth_view, encoder, self.config.clear_color);
        }

        if self.config.render_molecular_surface {
            self.molecular_surface_pass
                .render(view, depth_view, encoder, self.config.clear_color);
        }
    }

    /// On resize, the depth texture needs to be recreated.
    pub fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.resources.depth_texture = DepthTexture::new(device, config);
    }
}

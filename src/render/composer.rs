// TODO: Clean up imports

use crate::common::resources::CommonResources;

use super::{
    passes::{
        molecular_surface::{MolecularSurfacePass, MolecularSurfaceResources},
        spacefill::{SpacefillPass, SpacefillResources},
    },
    resources::{
        camera::CameraResource, depth_texture::DepthTexture,
        df_texture::DistanceFieldTextureRender, light::LightResource,
    },
};

pub struct RenderOwnedResources {
    pub depth_texture: DepthTexture,
    pub light_resource: LightResource,
    pub camera_resource: CameraResource,
    pub df_texture: DistanceFieldTextureRender,
}

/// Configuration for the renderer.
pub struct RenderConfig {
    /// Whether to render the spacefill representation.
    pub render_spacefill: bool,
    /// Whether to render the molecular surface representation.
    pub render_molecular_surface: bool,
    /// The clear color of the renderer.
    pub clear_color: wgpu::Color,
    // TODO: Add ArcballCamera?
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            render_spacefill: false,
            render_molecular_surface: true,
            clear_color: wgpu::Color::BLACK,
        }
    }
}

// TODO: Rename to RenderComposer
/// A collection of render passes that are executed in order to render the molecule.
pub struct RenderJobs {
    /// Configuration for the renderer. This is used to control what is rendered.
    pub config: RenderConfig,

    /// The resources required for rendering. TODO: Better docs.
    pub resources: RenderOwnedResources,

    spacefill_pass: SpacefillPass,
    molecular_surface_pass: MolecularSurfacePass,
}

impl RenderJobs {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        common: &CommonResources,
    ) -> RenderJobs {
        let resources = RenderOwnedResources {
            light_resource: LightResource::new(device),
            camera_resource: CameraResource::new(device),
            depth_texture: DepthTexture::new(device, config),
            df_texture: DistanceFieldTextureRender::new_with_resolution(device, 1), // TODO: Replace with some reasonabel constant
        };

        let spacefill_resources = SpacefillResources::new(&resources, common);
        let spacefill_pass = SpacefillPass::new(device, config, spacefill_resources);

        let molecular_surface_resources = MolecularSurfaceResources::new(&resources, common);
        let molecular_surface_pass =
            MolecularSurfacePass::new(device, config, molecular_surface_resources);

        Self {
            resources,
            spacefill_pass,
            molecular_surface_pass,
            config: RenderConfig::default(),
        }
    }

    pub fn execute(
        &mut self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        common: &CommonResources,
    ) {
        let depth_view = &self.resources.depth_texture.view;
        let spacefil_resources = SpacefillResources::new(&self.resources, common);

        if self.config.render_spacefill {
            self.spacefill_pass.render(
                view,
                depth_view,
                encoder,
                self.config.clear_color,
                spacefil_resources,
            );
        }

        if self.config.render_molecular_surface {
            let molecular_surface_resources =
                MolecularSurfaceResources::new(&self.resources, common);

            self.molecular_surface_pass.render(
                view,
                depth_view,
                encoder,
                self.config.clear_color,
                molecular_surface_resources,
            );
        }
    }

    /// On resize, the depth texture needs to be recreated.
    pub fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.resources.depth_texture = DepthTexture::new(device, config);
    }
}

// TODO: Clean up imports

use crate::common::{models::grid::GridUniform, resources::CommonResources};

use super::{
    passes::{
        molecular_surface::{MolecularSurfacePass, MolecularSurfaceResources},
        spacefill::{SpacefillPass, SpacefillResources},
    },
    resources::{
        camera::CameraResource, depth_texture::DepthTexture, distance_field::DistanceFieldRender,
        light::LightResource,
    },
};

pub struct RenderOwnedResources {
    // TODO: Merge this into a single struct
    pub distance_field: DistanceFieldRender,

    // TODO: Merge this into a single struct
    pub light_resource: LightResource,
    pub camera_resource: CameraResource,

    pub depth_texture: DepthTexture,
}

/// Configuration for the renderer.
pub struct RenderState {
    /// Whether to render the spacefill representation.
    pub render_spacefill: bool,
    /// Whether to render the molecular surface representation.
    pub render_molecular_surface: bool,
    /// The clear color of the renderer.
    pub clear_color: wgpu::Color,
    // TODO: Add ArcballCamera?
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

pub struct RenderParameters<'a> {
    pub common_resources: &'a CommonResources,
    pub surface_config: &'a wgpu::SurfaceConfiguration,
    pub render_spacefill: bool,
    pub render_molecular_surface: bool,
    pub clear_color: wgpu::Color,
}

// TODO: Rename to RenderComposer
/// A collection of render passes that are executed in order to render the molecule.
pub struct RenderJobs {
    /// Configuration for the renderer. This is used to control what is rendered.
    pub state: RenderState,

    /// The resources required for rendering. TODO: Better docs.
    pub resources: RenderOwnedResources,

    spacefill_pass: SpacefillPass,
    molecular_surface_pass: MolecularSurfacePass,
}

impl RenderJobs {
    pub fn new(device: &wgpu::Device, params: RenderParameters) -> RenderJobs {
        let state = RenderState::from(&params);

        let resources = RenderOwnedResources {
            light_resource: LightResource::new(device),
            camera_resource: CameraResource::new(device),
            depth_texture: DepthTexture::new(device, params.surface_config),
            distance_field: DistanceFieldRender::new(device, GridUniform::default()),
        };

        let spacefill_resources = SpacefillResources::new(&resources, params.common_resources);
        let spacefill_pass = SpacefillPass::new(device, params.surface_config, spacefill_resources);

        let molecular_surface_resources = MolecularSurfaceResources::new(&resources);
        let molecular_surface_pass =
            MolecularSurfacePass::new(device, params.surface_config, molecular_surface_resources);

        Self {
            state,
            resources,
            spacefill_pass,
            molecular_surface_pass,
        }
    }

    pub fn render(
        &mut self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        common: &CommonResources,
    ) {
        let depth_view = &self.resources.depth_texture.view;
        let spacefil_resources = SpacefillResources::new(&self.resources, common);

        if self.state.render_spacefill {
            self.spacefill_pass.render(
                view,
                depth_view,
                encoder,
                self.state.clear_color,
                spacefil_resources,
            );
        }

        if self.state.render_molecular_surface {
            let molecular_surface_resources = MolecularSurfaceResources::new(&self.resources);

            self.molecular_surface_pass.render(
                view,
                depth_view,
                encoder,
                self.state.clear_color,
                molecular_surface_resources,
            );
        }
    }

    pub fn update_distance_field_texture(
        &mut self,
        device: &wgpu::Device,
        texture: wgpu::Texture,
        grid: GridUniform,
    ) {
        self.resources.distance_field = DistanceFieldRender::from_texture(device, grid, texture);
    }

    /// On resize, the depth texture needs to be recreated.
    pub fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.resources.depth_texture = DepthTexture::new(device, config);
    }
}

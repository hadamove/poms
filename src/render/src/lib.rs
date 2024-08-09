mod passes;
// TODO: private
pub mod resources;
mod state;

use common::{models::grid::GridUniform, resources::CommonResources};

use passes::molecular_surface::{MolecularSurfacePass, MolecularSurfaceResources};
use passes::spacefill::{SpacefillPass, SpacefillResources};
use resources::{camera::CameraResource, light::LightResource};
use resources::{camera::CameraUniform, light::LightUniform};
use resources::{depth_texture::DepthTexture, distance_field::DistanceFieldRender};
use state::RenderState;

/// Contains all resources that are owned by the render pipeline.
pub struct RenderOwnedResources {
    pub distance_field: DistanceFieldRender,

    pub light_resource: LightResource,
    pub camera_resource: CameraResource,

    pub depth_texture: DepthTexture,
}

/// Manages the rendering of a molecule, so far two representations are supported:
/// - **Spacefill**: Atoms are represented as spheres.
/// - **Molecular Surface**: The surface of the molecule is rendered. Requires a distance field texture.
pub struct RenderJobs {
    /// Configuration for the renderer. This is used to control what is rendered.
    state: RenderState,
    resources: RenderOwnedResources,

    spacefill_pass: SpacefillPass,
    molecular_surface_pass: MolecularSurfacePass,
}

pub struct RenderParameters<'a> {
    pub common_resources: &'a CommonResources,
    pub surface_config: &'a wgpu::SurfaceConfiguration,
    pub render_spacefill: bool,
    pub render_molecular_surface: bool,
    pub clear_color: wgpu::Color,
    pub number_of_atoms: u32,
}

impl RenderJobs {
    /// Creates a new instance of `RenderJobs` with the given parameters.
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

    /// Records the enabled representations to the provided `encoder`.
    pub fn render(
        &mut self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        common: &CommonResources,
    ) {
        let depth_view = &self.resources.depth_texture.view;

        if self.state.render_spacefill {
            let spacefil_resources = SpacefillResources::new(&self.resources, common);

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

    /// Use this method to update the distance field texture upon completion of a compute pass.
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

    /// Enables or disables rendering of spacefill representation.
    pub fn toggle_spacefill(&mut self, is_enabled: bool) {
        self.state.render_spacefill = is_enabled;
    }

    /// Enables or disables rendering of molecular surface representation.
    pub fn toggle_molecular_surface(&mut self, is_enabled: bool) {
        self.state.render_molecular_surface = is_enabled;
    }

    /// Updates the clear color used by the render passes.
    /// Used to switch between light and dark mode.
    pub fn change_clear_color(&mut self, color: wgpu::Color) {
        self.state.clear_color = color;
    }

    /// Updates the view of the molecule.
    pub fn update_camera(&self, queue: &wgpu::Queue, camera_uniform: CameraUniform) {
        self.resources.camera_resource.update(queue, camera_uniform);
    }

    /// Updates the light uniform used to shade the molecule.
    pub fn update_light(&self, queue: &wgpu::Queue, light_uniform: LightUniform) {
        self.resources.light_resource.update(queue, light_uniform);
    }
}

mod passes;
mod resources;
mod state;

pub use passes::postprocess::PostprocessSettings;

use passes::molecular_surface::MolecularSurfacePass;
use passes::postprocess::PostprocessPass;
use passes::spacefill::SpacefillPass;
use resources::color_texture::ColorTexture;
use resources::depth_texture::DepthTexture;
use resources::distance_field::DistanceField;
use resources::normal_texture::NormalTexture;
use resources::{camera::CameraResource, light::LightResource};
use state::RenderSettings;

use poms_common::{models::grid::GridUniform, resources::CommonResources};

/// Contains all resources that are owned by the render pipeline.
pub struct RenderResources {
    pub distance_field: DistanceField,
    pub light: LightResource,
    pub camera: CameraResource,

    pub color_texture: ColorTexture,
    pub normal_texture: NormalTexture,
    pub depth_texture: DepthTexture,

    pub clear_color: wgpu::Color,
}

/// Manages the rendering of a molecule, so far two representations are supported:
/// - **Spacefill**: Atoms are represented as spheres.
/// - **Molecular Surface**: The surface of the molecule is rendered. Requires a distance field texture.
pub struct RenderJobs {
    /// Configuration for the renderer. This is used to control what is rendered.
    settings: RenderSettings,
    resources: RenderResources,

    spacefill_pass: SpacefillPass,
    molecular_surface_pass: MolecularSurfacePass,
    postprocess_pass: PostprocessPass,
}

/// Things required to create a new instance of `RenderJobs`.
pub struct RenderParameters<'a> {
    /// Resources shared between the compute and render pipelines. Contains molecule data on the GPU.
    pub common_resources: &'a CommonResources,
    /// Configuration of the surface to render to. Needed to create the depth texture and initialize the render passes.
    pub surface_config: &'a wgpu::SurfaceConfiguration,
    /// The queue used to submit commands to the GPU.
    pub queue: &'a wgpu::Queue,
    /// Flag to enable or disable rendering of the spacefill representation by default. May be changed by calling `toggle_spacefill`.
    pub render_spacefill: bool,
    /// Flag to enable or disable rendering of the molecular surface representation by default. May be changed by calling `toggle_molecular_surface`.
    pub render_molecular_surface: bool,
    /// Settings associated with postprocessing effects.
    pub postprocess_settings: PostprocessSettings,
    /// Clear color used by the render passes by default. May be changed by calling `change_clear_color`.
    pub clear_color: wgpu::Color,
}

impl RenderJobs {
    /// Creates a new instance of `RenderJobs` with the given parameters.
    pub fn new(device: &wgpu::Device, params: RenderParameters) -> RenderJobs {
        let state = RenderSettings::from(&params);

        let resources = RenderResources {
            light: LightResource::new(device),
            camera: CameraResource::new(device),
            color_texture: ColorTexture::new(device, params.surface_config),
            normal_texture: NormalTexture::new(device, params.surface_config),
            depth_texture: DepthTexture::new(device, params.surface_config),
            distance_field: DistanceField::new(device, GridUniform::default()),
            clear_color: wgpu::Color::BLACK,
        };

        let spacefill_pass = SpacefillPass::new(device, &resources, params.common_resources);
        let molecular_surface_pass = MolecularSurfacePass::new(device, &resources);
        let postprocess_pass = PostprocessPass::new(
            device,
            params.queue,
            params.surface_config,
            params.postprocess_settings,
            &resources,
        );

        Self {
            settings: state,
            resources,
            spacefill_pass,
            molecular_surface_pass,
            postprocess_pass,
        }
    }

    /// Records the enabled representations to the provided `encoder`.
    pub fn render(
        &mut self,
        output_texture_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        common_resources: &CommonResources,
    ) {
        if self.settings.render_spacefill {
            self.spacefill_pass
                .render(encoder, &self.resources, common_resources);
        }

        if self.settings.render_molecular_surface {
            self.molecular_surface_pass.render(encoder, &self.resources);
        }

        self.postprocess_pass
            .render(output_texture_view, encoder, &self.resources);
    }

    /// Use this method to update the distance field texture upon completion of a compute pass.
    pub fn update_distance_field_texture(
        &mut self,
        device: &wgpu::Device,
        texture: wgpu::Texture,
        grid: GridUniform,
    ) {
        self.resources.distance_field = DistanceField::from_texture(device, grid, texture);
    }

    /// On resize, all dependent textures needs to be recreated.
    pub fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.resources.color_texture = ColorTexture::new(device, config);
        self.resources.normal_texture = NormalTexture::new(device, config);
        self.resources.depth_texture = DepthTexture::new(device, config);

        self.postprocess_pass
            .resize(device, config, &self.resources);
    }

    /// Enables or disables rendering of spacefill representation.
    pub fn toggle_spacefill_pass(&mut self, is_enabled: bool) {
        self.settings.render_spacefill = is_enabled;
    }

    /// Enables or disables rendering of molecular surface representation.
    pub fn toggle_molecular_surface_pass(&mut self, is_enabled: bool) {
        self.settings.render_molecular_surface = is_enabled;
    }

    /// Changes parameters of the postprocessing effects (e.g. ssao).
    pub fn update_postprocess_settings(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        settings: PostprocessSettings,
    ) {
        if self.postprocess_pass.settings.ssao_samples_count != settings.ssao_samples_count {
            self.postprocess_pass.update_sampels_count(
                settings.ssao_samples_count,
                device,
                queue,
                &self.resources,
            );
        }
        self.postprocess_pass.settings = settings;
    }

    /// Updates the clear color used by the render passes.
    /// Used to switch between light and dark mode.
    pub fn update_clear_color(&mut self, color: wgpu::Color) {
        self.resources.clear_color = color;
    }

    /// Updates the camera uniform buffer with the new camera data.
    pub fn update_camera(
        &mut self,
        queue: &wgpu::Queue,
        position: cgmath::Point3<f32>,
        view_matrix: cgmath::Matrix4<f32>,
        projection_matrix: cgmath::Matrix4<f32>,
    ) {
        self.resources
            .camera
            .update(queue, position, view_matrix, projection_matrix);
        self.postprocess_pass
            .update_buffers(queue, projection_matrix);
    }

    /// Updates the light uniform used to shade the molecule.
    pub fn update_light(&self, queue: &wgpu::Queue, direction: cgmath::Vector3<f32>) {
        self.resources.light.update(queue, direction);
    }

    /// Returns the current state of molecular surface rendering. Used to determine whether or not trigger the compute pipeline.
    pub fn is_molecular_surface_pass_enabled(&self) -> bool {
        self.settings.render_molecular_surface
    }
}

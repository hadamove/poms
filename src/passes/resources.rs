pub mod atom;
pub mod camera;
pub mod grid;
pub mod light;
pub mod molecule;
pub mod textures;

use crate::context::Context;

use crate::utils::constants::MIN_SES_RESOLUTION;

use camera::{arcball::ArcballCamera, resource::CameraResource};
use grid::{molecule_grid::MoleculeGridResource, ses_grid::SesGridResource};
use textures::{depth_texture::DepthTexture, df_texture::DistanceFieldTexture};

use self::light::LightResource;

// TODO: : Clone (supertrait)
pub trait GpuResource {
    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout;
    fn bind_group(&self) -> &wgpu::BindGroup;
}

pub struct CommonResources {
    // TODO: move camera to input
    pub camera: ArcballCamera,

    // This makes sense here
    pub ses_resource: SesGridResource,
    pub molecule_resource: MoleculeGridResource,
    pub camera_resource: CameraResource,

    pub df_texture_front: DistanceFieldTexture,
    pub df_texture_back: DistanceFieldTexture,

    pub light_resource: LightResource,
    pub depth_texture: DepthTexture,
}

impl CommonResources {
    // TODO: replace with config and device
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        Self {
            camera: ArcballCamera::from_config(&config), // TODO: This has nothing to do here
            ses_resource: SesGridResource::new(&device),
            molecule_resource: MoleculeGridResource::new(&device),
            camera_resource: CameraResource::new(&device),

            df_texture_front: DistanceFieldTexture::new(&device, 1),
            df_texture_back: DistanceFieldTexture::new(&device, MIN_SES_RESOLUTION),

            depth_texture: DepthTexture::new(&device, &config),
            light_resource: LightResource::new(&device),
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.depth_texture = DepthTexture::new(&device, &config);
        self.camera.resize(&config);
    }

    pub fn get_depth_texture(&self) -> &DepthTexture {
        &self.depth_texture
    }
}

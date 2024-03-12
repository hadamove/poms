pub mod atom;
pub mod camera;
pub mod grid;
pub mod light;
pub mod molecule;
pub mod textures;

use crate::utils::constants::MIN_SES_RESOLUTION;

use camera::arcball::ArcballCamera;
use grid::{molecule_grid::MoleculeGridResource, ses_grid::SesGridResource};
use textures::df_texture::DistanceFieldTexture;

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

    // TODO: move these to compute & render respectively, figure out the swapping (maybe Arc<RwLock>)
    pub df_texture_back: DistanceFieldTexture,
    pub df_texture_front: DistanceFieldTexture,
}

impl CommonResources {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        Self {
            camera: ArcballCamera::from_config(config),
            ses_resource: SesGridResource::new(device),
            molecule_resource: MoleculeGridResource::new(device),
            df_texture_back: DistanceFieldTexture::new(device, MIN_SES_RESOLUTION),
            df_texture_front: DistanceFieldTexture::new(device, 1),
        }
    }

    pub fn resize(&mut self, _device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.camera.resize(config);
    }
}

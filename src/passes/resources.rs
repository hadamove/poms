pub mod atom;
pub mod camera;
pub mod grid;
pub mod light;
pub mod molecule;
pub mod textures;

use crate::utils::constants::MIN_SES_RESOLUTION;

use camera::arcball::ArcballCameraController;
use grid::{molecule_grid::MoleculeGridResource, ses_grid::SesGridResource};
use textures::df_texture::DistanceFieldTexture;

// TODO: : Clone (supertrait)
pub trait GpuResource {
    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout;
    fn bind_group(&self) -> &wgpu::BindGroup;
}

/// For efficiency, some resources (e.g. the molecule) are shared between render and compute passes.
pub struct CommonResources {
    // TODO: move camera to render?
    pub camera_controller: ArcballCameraController,

    // This makes sense here
    pub molecule_resource: MoleculeGridResource,
    pub ses_resource: SesGridResource, // TODO: This can probably be separate for render & compute

    // TODO: move these to compute & render respectively, figure out the swapping (maybe Arc<RwLock>)
    pub df_texture_back: DistanceFieldTexture,
    pub df_texture_front: DistanceFieldTexture,
}

impl CommonResources {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        Self {
            camera_controller: ArcballCameraController::from_config(config),
            ses_resource: SesGridResource::new(device),
            molecule_resource: MoleculeGridResource::new(device),
            df_texture_back: DistanceFieldTexture::new(device, MIN_SES_RESOLUTION),
            df_texture_front: DistanceFieldTexture::new(device, 1),
        }
    }

    pub fn resize(&mut self, _device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.camera_controller.resize(config);
    }
}

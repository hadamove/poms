pub mod atom;
pub mod camera;
pub mod grid;
pub mod light;
pub mod molecule;
pub mod textures;

use camera::arcball::ArcballCameraController;
use grid::{molecule_grid::MoleculeGridResource, ses_grid::SesGridResource};

/// For efficiency, some resources (e.g. the molecule) are shared between render and compute passes.
pub struct CommonResources {
    // TODO: move camera to render?
    pub camera_controller: ArcballCameraController,

    // This makes sense here
    pub molecule_resource: MoleculeGridResource,
    pub ses_resource: SesGridResource, // TODO: This can probably be separate for render & compute
}

impl CommonResources {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        Self {
            camera_controller: ArcballCameraController::from_config(config),
            ses_resource: SesGridResource::new(device),
            molecule_resource: MoleculeGridResource::new(device),
        }
    }

    pub fn resize(&mut self, _device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.camera_controller.resize(config);
    }
}

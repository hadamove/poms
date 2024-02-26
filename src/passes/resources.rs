pub mod atom;
pub mod camera;
pub mod grid;
pub mod light;
pub mod molecule;
pub mod textures;

use crate::context::Context;

use crate::utils::constants::MIN_SES_RESOLUTION;
use crate::utils::input::Input;

use camera::{arcball::ArcballCamera, resource::CameraResource};
use grid::{molecule_grid::MoleculeGridResource, ses_grid::SesGridResource};
use textures::{depth_texture::DepthTexture, df_texture::DistanceFieldTexture};

use self::light::LightResource;

// TODO: : Clone (supertrait)
pub trait GpuResource {
    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout;
    fn bind_group(&self) -> &wgpu::BindGroup;
}

pub struct ResourceRepo {
    // TODO: remove this
    pub just_switched: bool,

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

impl ResourceRepo {
    pub fn new(context: &Context) -> Self {
        Self {
            just_switched: false, // TODO: Just temp solution

            camera: ArcballCamera::from_config(&context.config), // TODO: This has nothing to do here
            ses_resource: SesGridResource::new(&context.device),
            molecule_resource: MoleculeGridResource::new(&context.device),
            camera_resource: CameraResource::new(&context.device),

            df_texture_front: DistanceFieldTexture::new(&context.device, 1),
            df_texture_back: DistanceFieldTexture::new(&context.device, MIN_SES_RESOLUTION),

            depth_texture: DepthTexture::new(&context.device, &context.config),
            light_resource: LightResource::new(&context.device),
        }
    }

    pub fn update(&mut self, context: &Context, input: &Input) {
        self.camera.update(input);
        self.camera_resource.update(&context.queue, &self.camera);
        self.light_resource
            .update_camera(&context.queue, &self.camera);
    }

    pub fn resize(&mut self, context: &Context) {
        self.depth_texture = DepthTexture::new(&context.device, &context.config);
        self.camera.resize(&context.config);
    }

    pub fn get_depth_texture(&self) -> &DepthTexture {
        &self.depth_texture
    }
}

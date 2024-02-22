pub mod camera;
pub mod grid;
pub mod light;
pub mod molecule;
pub mod molecule_repo;
pub mod textures;

use std::sync::Arc;

use crate::context::Context;
use crate::ui::event::UserEvent;
use crate::utils::constants::MIN_SES_RESOLUTION;
use crate::utils::input::Input;

use camera::{arcball::ArcballCamera, resource::CameraResource};
use grid::{molecule_grid::MoleculeGridResource, ses_grid::SesGridResource};
use molecule_repo::MoleculeRepo;
use textures::{depth_texture::DepthTexture, df_texture::DistanceFieldTexture};

use self::{grid::MoleculeWithNeighborGrid, light::LightResource};

use super::compute::ComputeProgress;

// TODO: : Clone (supertrait)
pub trait GpuResource {
    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout;
    fn bind_group(&self) -> &wgpu::BindGroup;
}

pub struct ResourceRepo {
    pub just_switched: bool,

    // TODO: This doesn't make sense here
    pub molecule_repo: MoleculeRepo,
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
        // let ses_state = SesState::default();

        Self {
            just_switched: false, // TODO: Just temp solution

            molecule_repo: MoleculeRepo::default(), // TODO: This has nothing to do here
            camera: ArcballCamera::from_config(&context.config), // TODO: This has nothing to do here
            // ses_state, // TODO: This has nothing to do here
            ses_resource: SesGridResource::new(&context.device),
            molecule_resource: MoleculeGridResource::new(&context.device),
            camera_resource: CameraResource::new(&context.device),

            df_texture_front: DistanceFieldTexture::new(&context.device, 1),
            df_texture_back: DistanceFieldTexture::new(&context.device, MIN_SES_RESOLUTION),

            depth_texture: DepthTexture::new(&context.device, &context.config),
            light_resource: LightResource::new(&context.device),
        }
    }

    // TODO: This should be handled somewhere in `app.rs` too
    pub fn update_compute_progress(
        &mut self,
        progress: ComputeProgress,
        context: &Context,
        molecule: Arc<MoleculeWithNeighborGrid>,
    ) {
        if let Some(render_resolution) = progress.last_computed_resolution {
            if render_resolution != self.df_texture_front.resolution() {
                let _front_resolution = self.df_texture_front.resolution();
                let _back_resolution = self.df_texture_back.resolution();
                // Swap textures
                self.df_texture_front = std::mem::replace(
                    &mut self.df_texture_back,
                    DistanceFieldTexture::new(&context.device, progress.current_resolution),
                );
                self.just_switched = true;
            }
        }
        self.ses_resource
            .update(&context.queue, &molecule.molecule, progress);
    }

    pub fn update(&mut self, context: &Context, input: &Input, gui_events: Vec<UserEvent>) {
        self.camera.update(input);
        self.camera_resource.update(&context.queue, &self.camera);
        self.light_resource
            .update_camera(&context.queue, &self.camera);
        self.handle_gui_events(context, gui_events);

        if let Some(mol) = self.molecule_repo.get_new() {
            self.camera.set_target(mol.molecule.calculate_center());
            self.molecule_resource
                .update(&context.queue, &mol.molecule, &mol.neighbor_grid);
        }
    }

    // TODO: `app` should handle the interaction between UI and resources
    fn handle_gui_events(&mut self, context: &Context, gui_events: Vec<UserEvent>) {
        for event in gui_events {
            #[allow(clippy::single_match)]
            match event {
                UserEvent::LoadedMolecules(molecules) => {
                    // TODO: Recreate ComputeJobs
                    self.molecule_repo.load_from_parsed(molecules, 1.4); // TODO: Replace constant
                }
                UserEvent::SesResolutionChanged(_resolution) => {
                    // TODO: Recreate ComputeJobs
                }
                UserEvent::ProbeRadiusChanged(probe_radius) => {
                    self.molecule_repo.recompute_neighbor_grids(probe_radius);
                    // TODO: Recreate ComputeJobs
                }
                UserEvent::ToggleAnimation => {
                    self.molecule_repo.toggle_animation();
                    // TODO: Recreate ComputeJobs?
                }
                UserEvent::AnimationSpeedChanged(speed) => {
                    self.molecule_repo.set_animation_speed(speed);
                }
                UserEvent::UpdateLight(light_data) => {
                    self.light_resource.update(&context.queue, light_data);
                }
                _ => {}
            }
        }
    }

    pub fn resize(&mut self, context: &Context) {
        self.depth_texture = DepthTexture::new(&context.device, &context.config);
        self.camera.resize(&context.config);
    }

    pub fn get_depth_texture(&self) -> &DepthTexture {
        &self.depth_texture
    }
}

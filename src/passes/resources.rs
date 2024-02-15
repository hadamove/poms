pub mod camera;
pub mod grid;
pub mod light;
pub mod molecule;
pub mod molecule_repo;
pub mod ses_state;
pub mod textures;

use crate::context::Context;
use crate::ui::event::UserEvent;
use crate::utils::constants::MIN_SES_RESOLUTION;
use crate::utils::input::Input;

use camera::{arcball::ArcballCamera, resource::CameraResource};
use grid::{molecule_grid::MoleculeGridResource, ses_grid::SesGridResource};
use molecule_repo::MoleculeRepo;
use ses_state::SesState;
use textures::{depth_texture::DepthTexture, df_texture::DistanceFieldTexture};

use self::grid::{GridSpacing, GridUniform};
use self::light::LightResource;
use self::ses_state::SesStage;

// TODO: : Clone
pub trait GpuResource {
    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout;
    fn bind_group(&self) -> &wgpu::BindGroup;
}

pub struct ResourceRepo {
    pub just_switched: bool,

    // TODO: This doesn't make sense here
    pub molecule_repo: MoleculeRepo,
    pub ses_state: SesState,
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
        let ses_state = SesState::default();

        Self {
            just_switched: false, // TODO: Just temp solution

            molecule_repo: MoleculeRepo::default(), // TODO: This has nothing to do here
            camera: ArcballCamera::from_config(&context.config), // TODO: This has nothing to do here
            ses_state, // TODO: This has nothing to do here

            ses_resource: SesGridResource::new(&context.device),
            molecule_resource: MoleculeGridResource::new(&context.device),
            camera_resource: CameraResource::new(&context.device),

            df_texture_front: DistanceFieldTexture::new(&context.device, MIN_SES_RESOLUTION),
            df_texture_back: DistanceFieldTexture::new(&context.device, MIN_SES_RESOLUTION),

            depth_texture: DepthTexture::new(&context.device, &context.config),
            light_resource: LightResource::new(&context.device),
        }
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
            self.reset_ses_stage(context);
        }

        self.increase_ses_frame(context);
    }

    fn increase_ses_frame(&mut self, context: &Context) {
        if let Some(molecule) = self.molecule_repo.get_current() {
            let ses_grid = GridUniform::from_molecule(
                &molecule.molecule,
                GridSpacing::Resolution(self.ses_state.get_compute_resolution()),
                self.ses_state.probe_radius,
            );

            self.ses_state.increase_frame(ses_grid.offset);
            self.ses_resource
                .update(&context.queue, &molecule.molecule, &self.ses_state);

            if self.ses_state.switch_ready() {
                self.df_texture_front = std::mem::replace(
                    &mut self.df_texture_back,
                    DistanceFieldTexture::new(
                        &context.device,
                        self.ses_state.get_compute_resolution(),
                    ),
                );
                self.ses_state.increase_frame(ses_grid.offset);
                self.molecule_repo.increase_frame();
                self.just_switched = true;
            }
        }
    }

    pub fn get_ses_stage(&self) -> &SesStage {
        &self.ses_state.stage
    }

    // TODO: `app` should handle the interaction between UI and resources
    fn handle_gui_events(&mut self, context: &Context, gui_events: Vec<UserEvent>) {
        for event in gui_events {
            #[allow(clippy::single_match)]
            match event {
                UserEvent::LoadedMolecules(molecules) => {
                    self.molecule_repo
                        .load_from_parsed(molecules, self.ses_state.probe_radius);
                }
                UserEvent::SesResolutionChanged(resolution) => {
                    self.ses_state.max_resolution = resolution;
                    self.reset_ses_stage(context);
                }
                UserEvent::ProbeRadiusChanged(probe_radius) => {
                    self.ses_state.probe_radius = probe_radius;
                    self.molecule_repo.recompute_neighbor_grids(probe_radius);
                    self.reset_ses_stage(context);
                }
                UserEvent::ToggleAnimation => {
                    self.molecule_repo.toggle_animation();
                    self.reset_ses_stage(context);
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

    fn reset_ses_stage(&mut self, context: &Context) {
        self.ses_state.stage = SesStage::Init;
        self.df_texture_back = DistanceFieldTexture::new(&context.device, MIN_SES_RESOLUTION);
    }

    pub fn resize(&mut self, context: &Context) {
        self.depth_texture = DepthTexture::new(&context.device, &context.config);
        self.camera.resize(&context.config);
    }

    pub fn get_num_grid_points(&self) -> u32 {
        self.ses_state.get_grid_points_count()
    }

    // pub fn get_num_atoms(&self) -> usize {
    //     match self.molecule_repo.get_current() {
    //         Some(molecule) => molecule.molecule.get_atoms().len(),
    //         None => 0,
    //     }
    // }

    pub fn get_depth_texture(&self) -> &DepthTexture {
        &self.depth_texture
    }
}

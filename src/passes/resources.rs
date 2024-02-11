mod camera;
mod grid;
mod light;
mod molecule;
mod molecule_repo;
pub mod ses_state;
mod textures;

use crate::context::Context;
use crate::gui::{GuiEvent, GuiEvents};
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

#[derive(Debug, PartialEq, Eq)]
pub enum PassId {
    ComputeProbe,
    ComputeRefinement,
    RenderSpacefill,
    RenderSesRaymarching,
}

pub trait Resource {
    fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout;
    fn get_bind_group(&self) -> &wgpu::BindGroup;
}

#[derive(Debug, Clone, Copy)]
pub struct GroupIndex(pub u32);

pub struct ResourceRepo {
    molecule_repo: MoleculeRepo,
    ses_state: SesState,

    ses_resource: SesGridResource,
    molecule_resource: MoleculeGridResource,
    camera_resource: CameraResource,
    camera: ArcballCamera,

    df_texture_front: DistanceFieldTexture,
    df_texture_back: DistanceFieldTexture,

    light_resource: LightResource,
    depth_texture: DepthTexture,
}

impl ResourceRepo {
    pub fn new(context: &Context) -> Self {
        let ses_state = SesState::default();

        Self {
            molecule_repo: MoleculeRepo::default(),
            ses_resource: SesGridResource::new(&context.device),

            molecule_resource: MoleculeGridResource::new(&context.device),
            camera_resource: CameraResource::new(&context.device),
            camera: ArcballCamera::from_config(&context.config),

            df_texture_front: DistanceFieldTexture::new(&context.device, MIN_SES_RESOLUTION),
            df_texture_back: DistanceFieldTexture::new(&context.device, MIN_SES_RESOLUTION),

            depth_texture: DepthTexture::new(&context.device, &context.config),
            light_resource: LightResource::new(&context.device),
            ses_state,
        }
    }

    pub fn update(&mut self, context: &Context, input: &Input, gui_events: GuiEvents) {
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
            }
        }
    }

    pub fn get_ses_stage(&self) -> &SesStage {
        &self.ses_state.stage
    }

    fn handle_gui_events(&mut self, context: &Context, gui_events: GuiEvents) {
        for event in gui_events {
            #[allow(clippy::single_match)]
            match event {
                GuiEvent::LoadedMolecules(molecules) => {
                    self.molecule_repo
                        .load_from_parsed(molecules, self.ses_state.probe_radius);
                }
                GuiEvent::SesResolutionChanged(resolution) => {
                    self.ses_state.max_resolution = resolution;
                    self.reset_ses_stage(context);
                }
                GuiEvent::ProbeRadiusChanged(probe_radius) => {
                    self.ses_state.probe_radius = probe_radius;
                    self.molecule_repo.recompute_neighbor_grids(probe_radius);
                    self.reset_ses_stage(context);
                }
                GuiEvent::ToggleAnimation => {
                    self.molecule_repo.toggle_animation();
                    self.reset_ses_stage(context);
                }
                GuiEvent::AnimationSpeedChanged(speed) => {
                    self.molecule_repo.set_animation_speed(speed);
                }
                GuiEvent::UpdateLight(light_data) => {
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

    pub fn get_num_atoms(&self) -> usize {
        match self.molecule_repo.get_current() {
            Some(molecule) => molecule.molecule.get_atoms().len(),
            None => 0,
        }
    }

    pub fn get_depth_texture(&self) -> &DepthTexture {
        &self.depth_texture
    }

    #[rustfmt::skip]
    pub fn get_resources(&self, pass_id: &PassId) -> ResourceGroup {
        match pass_id {
            PassId::ComputeProbe => ResourceGroup(vec![
                (GroupIndex(0), &self.ses_resource as &dyn Resource),
                (GroupIndex(1), &self.molecule_resource as &dyn Resource),
            ]),
            PassId::ComputeRefinement => ResourceGroup(vec![
                (GroupIndex(0), &self.ses_resource as &dyn Resource),
                (GroupIndex(1), &self.molecule_resource as &dyn Resource),
                (GroupIndex(2), &self.df_texture_back.compute as &dyn Resource),
            ]),
            PassId::RenderSpacefill => ResourceGroup(vec![
                (GroupIndex(0), &self.molecule_resource as &dyn Resource),
                (GroupIndex(1), &self.camera_resource as &dyn Resource),
            ]),
            PassId::RenderSesRaymarching => ResourceGroup(vec![
                (GroupIndex(0), &self.ses_resource as &dyn Resource),
                (GroupIndex(1), &self.df_texture_front.render as &dyn Resource),
                (GroupIndex(2), &self.camera_resource as &dyn Resource),
                (GroupIndex(3), &self.light_resource as &dyn Resource),
            ]),
        }
    }

    pub fn get_shader(pass_id: &PassId) -> wgpu::ShaderModuleDescriptor {
        match pass_id {
            PassId::ComputeProbe => wgpu::include_wgsl!("./shaders/probe.wgsl"),
            PassId::ComputeRefinement => wgpu::include_wgsl!("./shaders/df_refinement.wgsl"),
            PassId::RenderSpacefill => wgpu::include_wgsl!("./shaders/spacefill.wgsl"),
            PassId::RenderSesRaymarching => wgpu::include_wgsl!("./shaders/ses_raymarching.wgsl"),
        }
    }
}

pub struct ResourceGroup<'a>(Vec<(GroupIndex, &'a dyn Resource)>);

impl<'a> ResourceGroup<'a> {
    pub fn get_bind_groups(&self) -> Vec<(GroupIndex, &wgpu::BindGroup)> {
        self.0
            .iter()
            .map(|(index, resource)| (*index, resource.get_bind_group()))
            .collect()
    }

    pub fn get_bind_group_layouts(&self) -> Vec<&wgpu::BindGroupLayout> {
        self.0
            .iter()
            .map(|(_, resource)| resource.get_bind_group_layout())
            .collect()
    }
}

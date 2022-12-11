mod camera;
mod grid;
mod molecule;
mod molecule_repo;
pub mod ses_state;
mod textures;

use std::sync::Arc;

use crate::context::Context;
use crate::gui::{GuiEvent, GuiEvents};
use crate::utils::constants::DEFAULT_SES_RESOLUTION;
use crate::utils::input::Input;

use camera::{arcball::ArcballCamera, resource::CameraResource};
use grid::{molecule_grid::MoleculeGridResource, ses_grid::SesGridResource, GriddedMolecule};
use molecule::Molecule;
use molecule_repo::MoleculeRepo;
use ses_state::SesState;
use textures::{depth_texture::DepthTexture, df_texture::DistanceFieldTexture};

use self::ses_state::SesStage;

// TODO: move this into separate file.
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

            df_texture_front: DistanceFieldTexture::new(&context.device, DEFAULT_SES_RESOLUTION),
            df_texture_back: DistanceFieldTexture::new(&context.device, DEFAULT_SES_RESOLUTION),

            depth_texture: DepthTexture::new(&context.device, &context.config),
            ses_state,
        }
    }

    pub fn update(&mut self, context: &Context, input: &Input, gui_events: GuiEvents) {
        self.camera.update(input);
        self.camera_resource.update(&context.queue, &self.camera);
        self.handle_gui_events(context, gui_events);

        if let Some(new_molecule) = self.molecule_repo.get_new() {
            self.update_molecule(&context.queue, new_molecule);
            if self.ses_state.get_compute_resolution() != DEFAULT_SES_RESOLUTION {
                self.df_texture_back =
                    DistanceFieldTexture::new(&context.device, DEFAULT_SES_RESOLUTION);
            }
            self.ses_state.reset_stage();
        }

        self.increase_ses_frame(context);
    }

    fn increase_ses_frame(&mut self, context: &Context) {
        if let Some(molecule) = self.molecule_repo.get_current() {
            self.ses_state.increase_frame();
            self.ses_resource
                .update(&context.queue, &molecule, &self.ses_state);

            if self.ses_state.switch_ready() {
                self.df_texture_front = std::mem::replace(
                    &mut self.df_texture_back,
                    DistanceFieldTexture::new(
                        &context.device,
                        self.ses_state.get_compute_resolution(),
                    ),
                );
                self.ses_state.increase_frame();
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
                    self.ses_state.reset_stage();
                    self.df_texture_back =
                        DistanceFieldTexture::new(&context.device, DEFAULT_SES_RESOLUTION);
                }
                GuiEvent::ProbeRadiusChanged(probe_radius) => {
                    self.update_probe_radius(probe_radius);
                }
                GuiEvent::ToggleAnimation => {
                    self.molecule_repo.toggle_animation();
                    self.ses_state.reset_stage();
                    self.df_texture_back =
                        DistanceFieldTexture::new(&context.device, DEFAULT_SES_RESOLUTION);
                }
                _ => {}
            }
        }
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
            Some(molecule) => molecule.atoms_sorted.len(),
            None => 0,
        }
    }

    pub fn get_depth_texture(&self) -> &DepthTexture {
        &self.depth_texture
    }

    fn update_molecule(&mut self, queue: &wgpu::Queue, molecule: Arc<GriddedMolecule>) {
        self.camera
            .set_target(molecule.atoms_sorted.calculate_center());
        self.molecule_resource.update(queue, &molecule);
        self.ses_state.reset_stage();
    }

    fn update_probe_radius(&mut self, probe_radius: f32) {
        self.ses_state.probe_radius = probe_radius;
        self.molecule_repo.recompute_neighbor_grids(probe_radius);
        self.ses_state.reset_stage();
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

mod camera;
mod depth_texture;
mod distance_field;
mod molecule_grid;
mod ses_grid;

use std::sync::Arc;

use wgpu::{include_wgsl, ShaderModuleDescriptor};

use crate::compute::PassId;
use crate::gpu::GpuState;

use self::camera::CameraResource;
use self::depth_texture::DepthTexture;
use self::distance_field::DistanceFieldResource;
use self::molecule_grid::MoleculeGridResource;
use self::ses_grid::SesGridResource;

use super::camera::ArcballCamera;
use super::grid::GriddedMolecule;

// TODO: move this into separate file.
pub struct SesSettings {
    probe_radius: f32,
    resolution: u32,
}

impl Default for SesSettings {
    fn default() -> Self {
        Self {
            // TODO: use constants.
            probe_radius: 1.4,
            resolution: 64,
        }
    }
}

pub struct GlobalResources {
    molecule: Arc<GriddedMolecule>,
    ses_settings: SesSettings,

    ses_resource: SesGridResource,
    molecule_resource: MoleculeGridResource,
    distance_field_resource: DistanceFieldResource,
    depth_texture: DepthTexture,

    camera_resource: CameraResource,
}

pub trait Resource {
    fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout;
    fn get_bind_group(&self) -> &wgpu::BindGroup;
}

#[derive(Debug, Clone, Copy)]
pub struct GroupIndex(pub u32);

impl GlobalResources {
    pub fn new(gpu: &GpuState) -> Self {
        let ses_settings = SesSettings::default();

        Self {
            molecule: Arc::default(),
            ses_resource: SesGridResource::new(&gpu.device),
            molecule_resource: MoleculeGridResource::new(&gpu.device),
            distance_field_resource: DistanceFieldResource::new(
                &gpu.device,
                ses_settings.resolution,
            ),
            depth_texture: DepthTexture::new(&gpu.device, &gpu.config),
            camera_resource: CameraResource::new(&gpu.device),
            ses_settings,
        }
    }

    pub fn update_molecule(&mut self, queue: &wgpu::Queue, molecule: Arc<GriddedMolecule>) {
        self.molecule = molecule.clone();
        self.molecule_resource.update_molecule(queue, &molecule);
        self.ses_resource
            .update_grid(queue, &self.molecule, &self.ses_settings);
    }

    pub fn update_probe_radius(&mut self, queue: &wgpu::Queue, probe_radius: f32) {
        self.ses_settings.probe_radius = probe_radius;
        self.ses_resource
            .update_grid(queue, &self.molecule, &self.ses_settings);
    }

    // TODO: pass only GpuState here
    pub fn update_resolution(&mut self, gpu: &GpuState, resolution: u32) {
        self.ses_settings.resolution = resolution;
        self.ses_resource
            .update_grid(&gpu.queue, &self.molecule, &self.ses_settings);
        self.distance_field_resource =
            DistanceFieldResource::new(&gpu.device, self.ses_settings.resolution);
    }

    pub fn update_camera(&mut self, queue: &wgpu::Queue, camera: &ArcballCamera) {
        self.camera_resource.update(queue, camera);
    }

    pub fn resize(&mut self, gpu: &GpuState) {
        self.depth_texture = DepthTexture::new(&gpu.device, &gpu.config);
    }

    pub fn get_num_grid_points(&self) -> u32 {
        self.ses_settings.resolution.pow(3)
    }

    pub fn get_num_atoms(&self) -> u32 {
        self.molecule.atoms_sorted.len() as u32
    }

    pub fn get_depth_texture(&self) -> &DepthTexture {
        &self.depth_texture
    }

    #[rustfmt::skip]
    pub fn get_resources(&self, pass_id: &PassId) -> ResourceGroup {
        match pass_id {
            PassId::Probe => ResourceGroup(vec![
                (GroupIndex(0), &self.ses_resource as &dyn Resource),
                (GroupIndex(1), &self.molecule_resource as &dyn Resource),
            ]),
            PassId::DistanceFieldRefinement => ResourceGroup(vec![
                (GroupIndex(0), &self.ses_resource as &dyn Resource),
                (GroupIndex(1), &self.molecule_resource as &dyn Resource),
                (GroupIndex(2), &self.distance_field_resource.compute as &dyn Resource),
            ]),
            PassId::Spacefill => ResourceGroup(vec![
                (GroupIndex(0), &self.molecule_resource as &dyn Resource),
                (GroupIndex(1), &self.camera_resource as &dyn Resource),
            ]),
            PassId::SesRaymarching => ResourceGroup(vec![
                (GroupIndex(0), &self.ses_resource as &dyn Resource),
                (GroupIndex(1), &self.distance_field_resource.render as &dyn Resource),
                (GroupIndex(2), &self.camera_resource as &dyn Resource),
            ]),
        }
    }

    pub fn get_shader(pass_id: &PassId) -> ShaderModuleDescriptor {
        match pass_id {
            PassId::Probe => include_wgsl!("../shaders/probe.wgsl"),
            PassId::DistanceFieldRefinement => include_wgsl!("../shaders/df_refinement.wgsl"),
            PassId::Spacefill => include_wgsl!("../shaders/spacefill.wgsl"),
            PassId::SesRaymarching => include_wgsl!("../shaders/ses_raymarching.wgsl"),
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

mod camera;
mod depth_texture;
mod distance_field;
mod molecule_grid;
mod ses_grid;

use std::sync::Arc;

use crate::compute::PassId;

use self::camera::CameraResource;
use self::depth_texture::DepthTexture;
use self::distance_field::DistanceFieldResource;
use self::molecule_grid::MoleculeGridResource;
use self::ses_grid::SesGridResource;

use super::grid::MoleculeData;

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
    molecule: Arc<MoleculeData>,
    ses_settings: SesSettings,

    ses_resource: SesGridResource,
    molecule_resource: MoleculeGridResource,
    distance_field_resource: DistanceFieldResource,
    depth_texture: DepthTexture,

    // TODO: make this private
    pub camera_resource: CameraResource,
}

pub trait Resource {
    fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout;
    fn get_bind_group(&self) -> &wgpu::BindGroup;
}

#[derive(Debug, Clone, Copy)]
pub struct GroupIndex(pub u32);

impl GlobalResources {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let ses_settings = SesSettings::default();

        Self {
            molecule: Arc::default(),
            ses_resource: SesGridResource::new(device),
            molecule_resource: MoleculeGridResource::new(device),
            distance_field_resource: DistanceFieldResource::new(device, ses_settings.resolution),
            depth_texture: DepthTexture::new(device, config),
            camera_resource: CameraResource::new(device),
            ses_settings,
        }
    }

    pub fn update_molecule(&mut self, queue: &wgpu::Queue, molecule: Arc<MoleculeData>) {
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
    pub fn update_resolution(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        resolution: u32,
    ) {
        self.ses_settings.resolution = resolution;
        self.ses_resource
            .update_grid(queue, &self.molecule, &self.ses_settings);
        self.distance_field_resource =
            DistanceFieldResource::new(device, self.ses_settings.resolution);
    }

    pub fn resize(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.depth_texture = DepthTexture::new(device, &config);
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
            PassId::ProbePass => ResourceGroup(vec![
                (GroupIndex(0), &self.ses_resource as &dyn Resource),
                (GroupIndex(1), &self.molecule_resource as &dyn Resource),
            ]),
            PassId::DFRefinementPass => ResourceGroup(vec![
                (GroupIndex(0), &self.ses_resource as &dyn Resource),
                (GroupIndex(1), &self.molecule_resource as &dyn Resource),
                (GroupIndex(2), &self.distance_field_resource.compute as &dyn Resource),
            ]),
            PassId::SpacefillPass => ResourceGroup(vec![
                (GroupIndex(0), &self.molecule_resource as &dyn Resource),
                (GroupIndex(1), &self.camera_resource as &dyn Resource),
            ]),
            PassId::RaymarchPass => ResourceGroup(vec![
                (GroupIndex(0), &self.ses_resource as &dyn Resource),
                (GroupIndex(1), &self.distance_field_resource.render as &dyn Resource),
                (GroupIndex(2), &self.camera_resource as &dyn Resource),
            ]),
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

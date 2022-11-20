mod camera;
mod distance_field;
mod molecule_grid;
mod ses_grid;

use std::sync::Arc;

use self::camera::CameraResource;
use self::distance_field::DistanceFieldResource;
use self::molecule_grid::MoleculeGridResource;
use self::ses_grid::SesGridResource;

use super::grid::MoleculeData;

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
    pub camera_resource: CameraResource,
}

pub struct GroupIndex(pub u32);

impl GlobalResources {
    pub fn new(device: &wgpu::Device) -> Self {
        let ses_settings = SesSettings::default();

        Self {
            molecule: Arc::default(),
            ses_resource: SesGridResource::new(device),
            molecule_resource: MoleculeGridResource::new(device),
            distance_field_resource: DistanceFieldResource::new(device, ses_settings.resolution),
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

    pub fn update_resolution(
        &mut self,
        queue: &wgpu::Queue,
        device: &wgpu::Device,
        resolution: u32,
    ) {
        self.ses_settings.resolution = resolution;
        self.ses_resource
            .update_grid(queue, &self.molecule, &self.ses_settings);

        // Recreate the distance field texture
        self.distance_field_resource =
            DistanceFieldResource::new(device, self.ses_settings.resolution);
    }

    pub fn get_num_grid_points(&self) -> u32 {
        self.ses_settings.resolution.pow(3)
    }

    pub fn get_num_atoms(&self) -> u32 {
        self.molecule.atoms_sorted.len() as u32
    }

    pub fn probe_pass_bind_group_layouts(&self) -> [&wgpu::BindGroupLayout; 2] {
        [
            self.ses_resource.get_bind_group_layout(),
            self.molecule_resource.get_bind_group_layout(),
        ]
    }

    #[rustfmt::skip]
    pub fn probe_pass_bind_groups(&self) -> [(GroupIndex, &wgpu::BindGroup); 2] {
        [
            (GroupIndex(0), self.ses_resource.get_bind_group()),
            (GroupIndex(1), self.molecule_resource.get_bind_group()),
        ]
    }

    pub fn dfr_pass_bind_group_layouts(&self) -> [&wgpu::BindGroupLayout; 3] {
        [
            self.ses_resource.get_bind_group_layout(),
            self.molecule_resource.get_bind_group_layout(),
            self.distance_field_resource.get_compute_bind_group_layout(),
        ]
    }

    #[rustfmt::skip]
    pub fn dfr_pass_bind_groups(&self) -> [(GroupIndex, &wgpu::BindGroup); 3] {
        [
            (GroupIndex(0), self.ses_resource.get_bind_group()),
            (GroupIndex(1), self.molecule_resource.get_bind_group()),
            (GroupIndex(2), self.distance_field_resource.get_compute_bind_group()),
        ]
    }

    pub fn spacefill_pass_bind_group_layouts(&self) -> [&wgpu::BindGroupLayout; 2] {
        [
            self.molecule_resource.get_bind_group_layout(),
            self.camera_resource.get_bind_group_layout(),
        ]
    }

    pub fn spacefill_pass_bind_groups(&self) -> [(GroupIndex, &wgpu::BindGroup); 2] {
        [
            (GroupIndex(0), self.molecule_resource.get_bind_group()),
            (GroupIndex(1), self.camera_resource.get_bind_group()),
        ]
    }

    pub fn raymarch_pass_bind_group_layouts(&self) -> [&wgpu::BindGroupLayout; 3] {
        [
            self.ses_resource.get_bind_group_layout(),
            self.distance_field_resource.get_render_bind_group_layout(),
            self.camera_resource.get_bind_group_layout(),
        ]
    }

    #[rustfmt::skip]
    pub fn raymarch_pass_bind_groups(&self) -> [(GroupIndex, &wgpu::BindGroup); 3] {
        [
            (GroupIndex(0), self.ses_resource.get_bind_group()),
            (GroupIndex(1), self.distance_field_resource.get_render_bind_group()),
            (GroupIndex(2), self.camera_resource.get_bind_group()),
        ]
    }
}

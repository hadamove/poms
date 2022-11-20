use wgpu::util::DeviceExt;

use crate::shared::grid::{GridSpacing, GridUniform, MoleculeData};

use super::SesSettings;

pub struct SesGridResource {
    buffers: SesGridBuffers,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl SesGridResource {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffers = SesGridBuffers::new(device);
        let bind_group_layout =
            device.create_bind_group_layout(&SesGridBindGroup::LAYOUT_DESCRIPTOR);

        Self {
            bind_group: SesGridBindGroup::new(device, &buffers, &bind_group_layout).0,
            bind_group_layout,
            buffers,
        }
    }

    pub fn update_grid(
        &self,
        queue: &wgpu::Queue,
        molecule: &MoleculeData,
        ses_settings: &SesSettings,
    ) {
        let ses_grid = GridUniform::from_atoms(
            &molecule.atoms_sorted,
            GridSpacing::Resolution(ses_settings.resolution),
            ses_settings.probe_radius,
        );
        queue.write_buffer(
            &self.buffers.ses_grid_buffer,
            0,
            bytemuck::cast_slice(&[ses_grid]),
        );
    }

    pub fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

struct SesGridBuffers {
    ses_grid_buffer: wgpu::Buffer,
    probe_radius_buffer: wgpu::Buffer,
}

impl SesGridBuffers {
    fn new(device: &wgpu::Device) -> Self {
        let ses_grid_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ses Grid Uniform Buffer"),
            contents: bytemuck::cast_slice(&[GridUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let probe_radius_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Probe Radius Uniform Buffer"),
            contents: bytemuck::cast_slice(&[1.4f32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            ses_grid_buffer,
            probe_radius_buffer,
        }
    }
}

struct SesGridBindGroup(wgpu::BindGroup);

impl SesGridBindGroup {
    fn new(
        device: &wgpu::Device,
        buffers: &SesGridBuffers,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.ses_grid_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffers.probe_radius_buffer.as_entire_binding(),
                },
            ],
            label: Some("Shared Bind Group"),
        });

        Self(bind_group)
    }

    const LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'_> =
        wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("Shared Bind Group Layout"),
        };
}

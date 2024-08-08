use wgpu::util::DeviceExt;

const MAX_NUM_ATOMS: usize = 1_000_000;
const MAX_DISTANCE_FIELD_RESOLUTION: u32 = 256;
const MAX_NUM_GRID_POINTS: usize = u32::pow(MAX_DISTANCE_FIELD_RESOLUTION, 3) as usize;

use crate::models::{atom::AtomsWithLookup, grid::GridUniform};

// TODO: Divide this into atoms data and atoms lookup
pub struct AtomsWithLookupResource {
    atoms_data_buffer: wgpu::Buffer,
    atoms_lookup_grid_buffer: wgpu::Buffer,
    atoms_by_voxel_buffer: wgpu::Buffer,

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl AtomsWithLookupResource {
    pub fn new(device: &wgpu::Device) -> Self {
        let atoms_data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sorted Atoms Buffer"),
            contents: bytemuck::cast_slice(&[0u32; MAX_NUM_ATOMS]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let atoms_lookup_grid_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Neighbor Atoms Grid Uniform Buffer"),
                contents: bytemuck::cast_slice(&[GridUniform::default()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let atoms_by_voxel_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Segment by Voxel buffer"),
            contents: bytemuck::cast_slice(&[0u32; MAX_NUM_GRID_POINTS]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&LAYOUT_DESCRIPTOR);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: atoms_data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: atoms_lookup_grid_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: atoms_by_voxel_buffer.as_entire_binding(),
                },
            ],
            label: Some("Molecule Grid Bind Group"),
        });

        Self {
            atoms_data_buffer,
            atoms_lookup_grid_buffer,
            atoms_by_voxel_buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, atoms: &AtomsWithLookup) {
        queue.write_buffer(
            &self.atoms_data_buffer,
            0,
            bytemuck::cast_slice(atoms.data.as_slice()),
        );
        queue.write_buffer(
            &self.atoms_lookup_grid_buffer,
            0,
            bytemuck::cast_slice(&[atoms.atoms_lookup_grid]),
        );
        queue.write_buffer(
            &self.atoms_by_voxel_buffer,
            0,
            bytemuck::cast_slice(&atoms.atoms_by_voxel),
        );
    }
}

const LAYOUT_DESCRIPTOR: wgpu::BindGroupLayoutDescriptor<'static> =
    wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::all(),
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
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
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
        label: Some("Molecule Grid Bind Group Layout"),
    };

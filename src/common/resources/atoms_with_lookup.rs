use wgpu::util::DeviceExt;

use crate::app::constants::{MAX_NUM_ATOMS, MAX_NUM_GRID_POINTS};
use crate::common::models::{atom::AtomsWithLookup, grid::GridUniform};

pub struct AtomsWithLookupResource {
    buffers: AtomsWithLookupBuffers,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl AtomsWithLookupResource {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffers = AtomsWithLookupBuffers::new(device);
        let bind_group_layout =
            device.create_bind_group_layout(&AtomsWithLookupBindGroup::LAYOUT_DESCRIPTOR);

        let bind_group = AtomsWithLookupBindGroup::new(device, &buffers, &bind_group_layout).0;

        Self {
            buffers,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, atoms: &AtomsWithLookup) {
        queue.write_buffer(
            &self.buffers.atoms_data_buffer,
            0,
            bytemuck::cast_slice(atoms.data.as_slice()),
        );
        queue.write_buffer(
            &self.buffers.lookup_grid_buffer,
            0,
            bytemuck::cast_slice(&[atoms.lookup_grid]),
        );
        queue.write_buffer(
            &self.buffers.segment_by_voxel_buffer,
            0,
            bytemuck::cast_slice(&atoms.segment_by_voxel),
        );
    }
}

struct AtomsWithLookupBuffers {
    atoms_data_buffer: wgpu::Buffer,
    lookup_grid_buffer: wgpu::Buffer,
    segment_by_voxel_buffer: wgpu::Buffer,

    grid_point_class_buffer: wgpu::Buffer,
}

impl AtomsWithLookupBuffers {
    fn new(device: &wgpu::Device) -> Self {
        let atoms_sorted_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sorted Atoms Buffer"),
            contents: bytemuck::cast_slice(&[0u32; MAX_NUM_ATOMS]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let neighbor_grid_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Neighbor Atoms Grid Uniform Buffer"),
            contents: bytemuck::cast_slice(&[GridUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let grid_cells_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid cells buffer"),
            contents: bytemuck::cast_slice(&[0u32; MAX_NUM_GRID_POINTS]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let grid_point_class_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Grid point classification buffer"),
                contents: bytemuck::cast_slice(&vec![0u32; MAX_NUM_GRID_POINTS]),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

        Self {
            atoms_data_buffer: atoms_sorted_buffer,
            lookup_grid_buffer: neighbor_grid_buffer,
            segment_by_voxel_buffer: grid_cells_buffer,
            grid_point_class_buffer,
        }
    }
}

struct AtomsWithLookupBindGroup(wgpu::BindGroup);

impl AtomsWithLookupBindGroup {
    fn new(
        device: &wgpu::Device,
        buffers: &AtomsWithLookupBuffers,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.atoms_data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffers.lookup_grid_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffers.segment_by_voxel_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buffers.grid_point_class_buffer.as_entire_binding(),
                },
            ],
            label: Some("Molecule Grid Bind Group"),
        });
        Self(bind_group)
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
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("Molecule Grid Bind Group Layout"),
        };
}

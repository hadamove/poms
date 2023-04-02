use wgpu::util::DeviceExt;

use crate::passes::resources::molecule::Molecule;
use crate::utils::constants::{MAX_NUM_ATOMS, MAX_NUM_GRID_POINTS};

use super::super::Resource;
use super::{GridUniform, NeighborGrid};

pub struct MoleculeGridResource {
    buffers: MoleculeGridBuffers,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl MoleculeGridResource {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffers = MoleculeGridBuffers::new(device);
        let bind_group_layout =
            device.create_bind_group_layout(&MoleculeGridBindGroup::LAYOUT_DESCRIPTOR);

        Self {
            bind_group: MoleculeGridBindGroup::new(device, &buffers, &bind_group_layout).0,
            bind_group_layout,
            buffers,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, molecule: &Molecule, grid: &NeighborGrid) {
        queue.write_buffer(
            &self.buffers.atoms_sorted_buffer,
            0,
            bytemuck::cast_slice(molecule.get_atoms()),
        );
        queue.write_buffer(
            &self.buffers.neighbor_grid_buffer,
            0,
            bytemuck::cast_slice(&[grid.uniform]),
        );
        queue.write_buffer(
            &self.buffers.grid_cells_buffer,
            0,
            bytemuck::cast_slice(&grid.grid_cells),
        );
    }

    pub fn clear_predecessor_buffer(&self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.buffers.predecessor_buffer,
            0,
            bytemuck::cast_slice(&vec![0u32; MAX_NUM_GRID_POINTS]),
        );
    }
}

impl Resource for MoleculeGridResource {
    fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

struct MoleculeGridBuffers {
    atoms_sorted_buffer: wgpu::Buffer,
    neighbor_grid_buffer: wgpu::Buffer,
    grid_cells_buffer: wgpu::Buffer,

    grid_point_class_buffer: wgpu::Buffer,
    predecessor_buffer: wgpu::Buffer,
}

impl MoleculeGridBuffers {
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
                usage: wgpu::BufferUsages::STORAGE,
            });

        let predecessor_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Predecessor buffer"),
            contents: bytemuck::cast_slice(&vec![0u32; MAX_NUM_GRID_POINTS]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            atoms_sorted_buffer,
            neighbor_grid_buffer,
            grid_cells_buffer,
            grid_point_class_buffer,
            predecessor_buffer,
        }
    }
}

struct MoleculeGridBindGroup(wgpu::BindGroup);

impl MoleculeGridBindGroup {
    fn new(
        device: &wgpu::Device,
        buffers: &MoleculeGridBuffers,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.atoms_sorted_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffers.neighbor_grid_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffers.grid_cells_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buffers.grid_point_class_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: buffers.predecessor_buffer.as_entire_binding(),
                },
            ],
            label: Some("Molecule Grid Bind Group"),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
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

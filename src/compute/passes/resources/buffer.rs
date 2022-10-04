use wgpu::util::DeviceExt;

use crate::compute::grid::{NeighborAtomGrid, SESGrid};

pub const MAX_NUM_GRID_POINTS: usize = usize::pow(256, 3);

pub struct ProbePassBuffers {
    pub neighbor_atom_grid_buffer: wgpu::Buffer,
    pub sorted_atoms_buffer: wgpu::Buffer,
    // TODO: refactor these buffers
    pub grid_cell_start_buffer: wgpu::Buffer,
    pub grid_cell_size_buffer: wgpu::Buffer,
}

impl ProbePassBuffers {
    pub fn new(device: &wgpu::Device, neighbor_atom_grid: &NeighborAtomGrid) -> Self {
        let neighbor_atom_grid_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Neighbor Atoms Grid Uniform Buffer"),
                contents: bytemuck::cast_slice(&[neighbor_atom_grid.uniform]),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let sorted_atoms_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sorted Atoms Buffer"),
            contents: bytemuck::cast_slice(&neighbor_atom_grid.sorted_atoms),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let grid_cell_start_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid cell start buffer"),
            contents: bytemuck::cast_slice(&neighbor_atom_grid.grid_cell_start),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let grid_cell_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid cell size buffer"),
            contents: bytemuck::cast_slice(&neighbor_atom_grid.grid_cell_size),
            usage: wgpu::BufferUsages::STORAGE,
        });

        Self {
            neighbor_atom_grid_buffer,
            sorted_atoms_buffer,
            grid_cell_start_buffer,
            grid_cell_size_buffer,
        }
    }
}

pub struct SharedBuffers {
    pub ses_grid_buffer: wgpu::Buffer,
    pub grid_point_classification_buffer: wgpu::Buffer,
}

impl SharedBuffers {
    pub fn new(device: &wgpu::Device, ses_grid: &SESGrid) -> Self {
        let ses_grid_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("SES Grid Uniform Buffer"),
            contents: bytemuck::cast_slice(&[ses_grid.uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let grid_point_classification_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Grid point classification buffer"),
                contents: bytemuck::cast_slice(&vec![0u32; MAX_NUM_GRID_POINTS]),
                usage: wgpu::BufferUsages::STORAGE,
            });

        Self {
            ses_grid_buffer,
            grid_point_classification_buffer,
        }
    }
}

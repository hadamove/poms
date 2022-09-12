use wgpu::util::DeviceExt;

use crate::{
    grid::{NeighborAtomGrid, SESGrid},
    parser::Atom,
};

pub struct ProbeComputePassBuffers {
    pub ses_grid_buffer: wgpu::Buffer,
    pub neighbor_atom_grid_buffer: wgpu::Buffer,

    pub grid_points_buffer: wgpu::Buffer,
    pub sorted_atoms_buffer: wgpu::Buffer,
    pub grid_cell_start_buffer: wgpu::Buffer,
    pub grid_cell_size_buffer: wgpu::Buffer,
}

impl ProbeComputePassBuffers {
    pub fn new(
        device: &wgpu::Device,
        ses_grid: &SESGrid,
        neighbor_atom_grid: &NeighborAtomGrid,
    ) -> Self {
        let ses_grid_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("SES Grid Uniform Buffer"),
            contents: bytemuck::cast_slice(&[ses_grid.grid]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let neighbor_atom_grid_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Neighbor Atoms Grid Uniform Buffer"),
                contents: bytemuck::cast_slice(&[neighbor_atom_grid.grid]),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let num_grid_points = ses_grid.get_num_grid_points();
        let initial_grid_points = vec![Atom::default(); num_grid_points as usize];

        let grid_points_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Points Buffer"),
            contents: bytemuck::cast_slice(&initial_grid_points),
            usage: wgpu::BufferUsages::STORAGE,
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
            ses_grid_buffer,
            neighbor_atom_grid_buffer,
            grid_points_buffer,
            sorted_atoms_buffer,
            grid_cell_start_buffer,
            grid_cell_size_buffer,
        }
    }
}

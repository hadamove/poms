use wgpu::util::DeviceExt;

use crate::compute::grid::NeighborAtomGrid;

pub const MAX_NUM_GRID_POINTS: usize = usize::pow(256, 3);

pub struct ProbePassBuffers {
    // Input buffers
    pub neighbor_atom_grid_buffer: wgpu::Buffer,
    pub atoms_sorted_by_grid_cells_buffer: wgpu::Buffer,
    pub grid_cells_buffer: wgpu::Buffer,

    // Output buffer
    pub grid_point_class_buffer: wgpu::Buffer,
}

impl ProbePassBuffers {
    pub fn new(device: &wgpu::Device, neighbor_atom_grid: &NeighborAtomGrid) -> Self {
        let neighbor_atom_grid_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Neighbor Atoms Grid Uniform Buffer"),
                contents: bytemuck::cast_slice(&[neighbor_atom_grid.uniform]),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let atoms_sorted_by_grid_cells_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Sorted Atoms Buffer"),
                contents: bytemuck::cast_slice(&neighbor_atom_grid.atoms_sorted_by_grid_cells),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let grid_cells_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid cells buffer"),
            contents: bytemuck::cast_slice(&neighbor_atom_grid.grid_cells),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let grid_point_class_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Grid point classification buffer"),
                contents: bytemuck::cast_slice(&vec![0u32; MAX_NUM_GRID_POINTS]),
                usage: wgpu::BufferUsages::STORAGE,
            });

        Self {
            neighbor_atom_grid_buffer,
            atoms_sorted_by_grid_cells_buffer,
            grid_cells_buffer,
            grid_point_class_buffer,
        }
    }
}

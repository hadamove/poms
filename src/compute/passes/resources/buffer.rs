use wgpu::util::DeviceExt;

use crate::shared::grid::{GridUniform, MoleculeData};

pub const MAX_NUM_ATOMS: usize = 1_000_000;
pub const MAX_NUM_GRID_POINTS: usize = usize::pow(256, 3);

pub struct ProbePassBuffers {
    // Input buffers
    pub neighbor_grid_buffer: wgpu::Buffer,
    pub atoms_sorted_buffer: wgpu::Buffer,
    pub grid_cells_buffer: wgpu::Buffer,

    // Output buffer
    pub grid_point_class_buffer: wgpu::Buffer,
}

impl ProbePassBuffers {
    pub fn new(device: &wgpu::Device) -> Self {
        let neighbor_grid_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Neighbor Atoms Grid Uniform Buffer"),
            contents: bytemuck::cast_slice(&[GridUniform::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let atoms_sorted_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sorted Atoms Buffer"),
            contents: bytemuck::cast_slice(&[0u32; MAX_NUM_ATOMS]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
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

        Self {
            neighbor_grid_buffer,
            atoms_sorted_buffer,
            grid_cells_buffer,
            grid_point_class_buffer,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, molecule: &MoleculeData) {
        queue.write_buffer(
            &self.neighbor_grid_buffer,
            0,
            bytemuck::cast_slice(&[molecule.neighbor_grid]),
        );

        queue.write_buffer(
            &self.atoms_sorted_buffer,
            0,
            bytemuck::cast_slice(&molecule.atoms_sorted),
        );

        queue.write_buffer(
            &self.grid_cells_buffer,
            0,
            bytemuck::cast_slice(&molecule.grid_cells),
        );
    }
}

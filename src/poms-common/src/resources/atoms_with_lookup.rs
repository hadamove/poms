use wgpu::util::DeviceExt;

use crate::limits::{MAX_NUM_ATOMS, MAX_NUM_GRID_POINTS};
use crate::models::atom::{Atom, AtomsWithLookup};
use crate::models::grid::GridUniform;

/// Contains buffers with atoms data and lookup grid for neighbor atoms.
pub struct AtomsWithLookupResource {
    atoms_data_buffer: wgpu::Buffer,
    atoms_lookup_grid_buffer: wgpu::Buffer,
    atoms_by_voxel_buffer: wgpu::Buffer,

    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,

    /// Number of atoms in the molecule.
    /// This is used to determine how many instances to render for the spacefill representation.
    pub number_of_atoms: u32,
}

impl AtomsWithLookupResource {
    /// Creates a new instance of `AtomsWithLookupResource`.
    /// Buffers of size `MAX_NUM_ATOMS` and `MAX_NUM_GRID_POINTS` are created to avoid resizing them later.
    pub fn new(device: &wgpu::Device) -> Self {
        let atoms_data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sorted Atoms Buffer"),
            contents: bytemuck::cast_slice(&[0u32; MAX_NUM_ATOMS * std::mem::size_of::<Atom>()]),
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
            number_of_atoms: 0,
        }
    }

    /// Updates the buffers with the provided atoms data and lookup grid.
    pub fn update(&mut self, queue: &wgpu::Queue, atoms: &AtomsWithLookup) {
        self.number_of_atoms = atoms.data.len() as u32;
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

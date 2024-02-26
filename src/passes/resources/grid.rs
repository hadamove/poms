use crate::utils::constants::MAX_PROBE_RADIUS;

use super::atom::*;
use cgmath::{Point3, Vector3, Vector4};

pub mod molecule_grid;
pub mod ses_grid;

// TODO: create an intermediate struct between app logic and uniforms, same for light data
/// TODO: better docs
// TODO: rename to UniformGrid
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GridUniform {
    /// The point in space where the grid starts (the minimum x, y, z coordinates).
    origin: [f32; 4],
    /// Number of grid points in each direction.
    pub resolution: u32,
    /// Step size and stuff TODO: rename to `spacing`
    offset: f32,
    size: f32, // TODO: Remove size it's unused
    // Add 4 bytes padding to avoid alignment issues.
    _padding: [u8; 4],
}

/// This struct is used to store the first atom index and the number of atoms in each voxel of the neighbor lookup grid.
/// Since the atoms are sorted by voxel index, we can use this to quickly find all atoms in a voxel.
#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AtomsSegment {
    pub first_atom_index: u32,
    pub atoms_count: u32,
}

/// Data structure that helps us faciliate fast look up of neighbor atoms in a molecule.
#[derive(Debug, Default)]
pub struct AtomsWithLookup {
    /// Atoms in the molecule, sorted by index of voxel they occupy for fast neighbor look up.
    pub data: Vec<Atom>,
    /// The grid that divdes the space around the molecule into neighborhood voxels. Usually, the spacing is equal to the probe radius plus the maximum atom radius.
    pub lookup_grid: GridUniform,
    /// Segment of atoms for each voxel of the neighbor lookup grid. The length of this vector is equal to the number of voxels in the grid (resolution^3).
    pub segment_by_voxel: Vec<AtomsSegment>,
}

impl AtomsWithLookup {
    pub fn new(atoms: Vec<Atom>, probe_radius: f32) -> Self {
        let lookup_grid = create_neighbor_lookup_grid_around_molecule(&atoms, probe_radius);
        // Extend atom data with corresponding voxel index
        let mut atoms_with_voxel_indices = atoms
            .into_iter()
            .map(|atom| {
                (
                    atom,
                    position_to_voxel_index(cgmath::Point3::from(atom.position), &lookup_grid),
                )
            })
            .collect::<Vec<_>>();
        // Sort atoms by the index of corresponding voxel
        atoms_with_voxel_indices.sort_by(|(_, i), (_, j)| i.cmp(j));

        // Create a look-up table for each voxel in the grid.
        let voxels_count = u32::pow(lookup_grid.resolution, 3) as usize;
        let mut segment_by_voxel = vec![AtomsSegment::default(); voxels_count];

        // Assign first index and count the number of atoms in each voxel
        for (atom_index, &(_, voxel_index)) in atoms_with_voxel_indices.iter().enumerate() {
            if segment_by_voxel[voxel_index].atoms_count == 0 {
                segment_by_voxel[voxel_index].first_atom_index = atom_index as u32;
            }
            segment_by_voxel[voxel_index].atoms_count += 1;
        }

        // Strip voxel indices from atoms
        let data = atoms_with_voxel_indices
            .into_iter()
            .map(|(atom, _)| atom)
            .collect();

        Self {
            data,
            lookup_grid,
            segment_by_voxel,
        }
    }
}

pub fn create_compute_grid_around_molecule(atoms: &[Atom], resolution: u32) -> GridUniform {
    let max_atom_radius = get_max_atom_radius(atoms);
    let margin = 2.0 * MAX_PROBE_RADIUS + max_atom_radius;

    let origin = get_min_position(atoms) - margin * Vector3::from((1.0, 1.0, 1.0));
    let size = get_max_distance(atoms) + 2.0 * margin;
    let offset = size / resolution as f32;

    GridUniform {
        origin: origin.to_homogeneous().into(),
        resolution,
        offset,
        size,
        _padding: Default::default(),
    }
}

pub fn create_neighbor_lookup_grid_around_molecule(
    atoms: &[Atom],
    probe_radius: f32,
) -> GridUniform {
    let max_atom_radius = get_max_atom_radius(atoms);
    let margin = 2.0 * MAX_PROBE_RADIUS + max_atom_radius;

    let origin = get_min_position(atoms) - margin * Vector3::from((1.0, 1.0, 1.0));
    let size = get_max_distance(atoms) + 2.0 * margin;
    let offset = probe_radius + max_atom_radius;

    let resolution = (size / offset).ceil() as u32;

    GridUniform {
        origin: origin.to_homogeneous().into(),
        offset,
        resolution,
        size,
        _padding: Default::default(),
    }
}

pub fn position_to_voxel_index(position: Point3<f32>, grid: &GridUniform) -> usize {
    let grid_origin = Vector4::from(grid.origin).truncate();
    let Point3 { x, y, z } = (position - grid_origin) / grid.offset;
    let r = grid.resolution as usize;
    (x as usize) + (y as usize * r) + (z as usize * r * r)
}

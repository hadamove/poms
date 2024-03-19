use cgmath::{Bounded, Point3, Vector3};

use super::grid::{create_atoms_lookup_grid_around_molecule, position_to_voxel_index, GridUniform};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Atom {
    pub position: [f32; 3],
    pub radius: f32,
    pub color: [f32; 4],
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
    pub atoms_lookup_grid: GridUniform,
    /// Segment of atoms for each voxel of the neighbor lookup grid. The length of this vector is equal to the number of voxels in the grid (resolution^3).
    pub atoms_by_voxel: Vec<AtomsSegment>,
}

impl AtomsWithLookup {
    pub fn new(atoms: Vec<Atom>, probe_radius: f32) -> Self {
        let atoms_lookup_grid = create_atoms_lookup_grid_around_molecule(&atoms, probe_radius);
        // Extend atom data with corresponding voxel index
        let mut atoms_with_voxel_indices = atoms
            .into_iter()
            .map(|atom| {
                (
                    atom,
                    position_to_voxel_index(
                        cgmath::Point3::from(atom.position),
                        &atoms_lookup_grid,
                    ),
                )
            })
            .collect::<Vec<_>>();
        // Sort atoms by the index of corresponding voxel
        atoms_with_voxel_indices.sort_by(|(_, i), (_, j)| i.cmp(j));

        // Create a look-up table for each voxel in the grid.
        let voxels_count = u32::pow(atoms_lookup_grid.resolution, 3) as usize;
        let mut atoms_by_voxel = vec![AtomsSegment::default(); voxels_count];

        // Assign first index and count the number of atoms in each voxel
        for (atom_index, &(_, voxel_index)) in atoms_with_voxel_indices.iter().enumerate() {
            if atoms_by_voxel[voxel_index].atoms_count == 0 {
                atoms_by_voxel[voxel_index].first_atom_index = atom_index as u32;
            }
            atoms_by_voxel[voxel_index].atoms_count += 1;
        }

        // Strip voxel indices from atoms
        let data = atoms_with_voxel_indices
            .into_iter()
            .map(|(atom, _)| atom)
            .collect();

        Self {
            data,
            atoms_lookup_grid,
            atoms_by_voxel,
        }
    }
}
pub fn calculate_center(atoms: &[Atom]) -> Point3<f32> {
    let mut center = Point3::new(0.0, 0.0, 0.0);
    for atom in atoms.iter() {
        center += Vector3::from(atom.position);
    }
    center / atoms.len() as f32
}

pub fn get_max_distance(atoms: &[Atom]) -> f32 {
    let min = get_min_position(atoms);
    let max = get_max_position(atoms);
    f32::max(max.x - min.x, f32::max(max.y - min.y, max.z - min.z))
}

pub fn get_max_atom_radius(atoms: &[Atom]) -> f32 {
    atoms.iter().map(|a| a.radius).fold(0.0, f32::max)
}

pub fn get_max_position(atoms: &[Atom]) -> Point3<f32> {
    atoms.iter().fold(Point3::min_value(), |res, atom| {
        let position = Point3::from(atom.position);
        Point3::new(
            f32::max(position.x, res.x),
            f32::max(position.y, res.y),
            f32::max(position.z, res.z),
        )
    })
}

pub fn get_min_position(atoms: &[Atom]) -> Point3<f32> {
    atoms.iter().fold(Point3::max_value(), |res, atom| {
        let position = Point3::from(atom.position);
        Point3::new(
            f32::min(position.x, res.x),
            f32::min(position.y, res.y),
            f32::min(position.z, res.z),
        )
    })
}

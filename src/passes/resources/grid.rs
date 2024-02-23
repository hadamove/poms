use crate::utils::constants::MAX_PROBE_RADIUS;

use super::molecule::Molecule;
use cgmath::{Point3, Vector3, Vector4};

pub mod molecule_grid;
pub mod ses_grid;

// TODO: create an intermediate struct between app logic and uniforms, same for light data
/// TODO: better docs
#[repr(C)]
#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GridUniform {
    /// The point in space where the grid starts (the minimum x, y, z coordinates).
    origin: [f32; 4],
    /// Number of grid points in each direction.
    resolution: u32,
    /// Step size and stuff TODO: rename to `spacing`
    offset: f32,
    size: f32, // TODO: Remove size it's unused
    // Add 4 bytes padding to avoid alignment issues.
    _padding: [u8; 4],
}

// TODO: Come up with better semantics and name for this
#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GridCell {
    first_atom_index: u32,
    atoms_count: u32,
}

// Data structure for efficient lookup of neighboring atoms.
#[derive(Debug, Default)]
pub struct NeighborGrid {
    pub uniform: GridUniform,
    pub grid_cells: Vec<GridCell>,
}

// TODO: Move this elsewhere
pub struct MoleculeWithNeighborGrid {
    pub molecule: Molecule,
    // This is updated when the molecule changes or probe radius is changed. In which case it's sent to the GPU
    pub neighbor_grid: NeighborGrid,
}

impl MoleculeWithNeighborGrid {
    pub fn from_molecule(molecule: &Molecule, probe_radius: f32) -> Self {
        let neighbor_grid_uniform =
            create_neighbor_lookup_grid_around_molecule(molecule, probe_radius);

        let atoms = molecule.get_atoms();
        // Divide atoms into grid cells for constant look up.
        let grid_cell_indices = atoms
            .iter()
            .map(|atom| compute_grid_cell_index(atom.get_position(), &neighbor_grid_uniform))
            .collect::<Vec<_>>();

        // Sort the atoms by cell index.
        let permutation = permutation::sort(&grid_cell_indices);
        let atoms_sorted = permutation.apply_slice(atoms);
        let grid_cell_indices = permutation.apply_slice(&grid_cell_indices);

        let grid_cell_count = u32::pow(neighbor_grid_uniform.resolution, 3) as usize;
        let mut grid_cells = vec![GridCell::default(); grid_cell_count];

        // Compute grid cell start indices and size in the atoms vector
        for (atom_index, &cell_index) in grid_cell_indices.iter().enumerate() {
            if grid_cells[cell_index].atoms_count == 0 {
                grid_cells[cell_index].first_atom_index = atom_index as u32;
            }
            grid_cells[cell_index].atoms_count += 1;
        }

        Self {
            molecule: Molecule::new(atoms_sorted),
            neighbor_grid: NeighborGrid {
                uniform: neighbor_grid_uniform,
                grid_cells,
            },
        }
    }
}

pub fn create_compute_grid_around_molecule(molecule: &Molecule, resolution: u32) -> GridUniform {
    let max_atom_radius = molecule.get_max_atom_radius();
    let margin = 2.0 * MAX_PROBE_RADIUS + max_atom_radius;

    let origin = molecule.get_min_position() - margin * Vector3::from((1.0, 1.0, 1.0));
    let size = molecule.get_max_distance() + 2.0 * margin;
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
    molecule: &Molecule,
    probe_radius: f32,
) -> GridUniform {
    let max_atom_radius = molecule.get_max_atom_radius();
    let margin = 2.0 * MAX_PROBE_RADIUS + max_atom_radius;

    let origin = molecule.get_min_position() - margin * Vector3::from((1.0, 1.0, 1.0));
    let size = molecule.get_max_distance() + 2.0 * margin;

    let offset = probe_radius + max_atom_radius;

    let size = molecule.get_max_distance() + 2.0 * margin;
    let resolution = (size / offset).ceil() as u32;

    GridUniform {
        origin: origin.to_homogeneous().into(),
        offset,
        resolution,
        size,
        _padding: Default::default(),
    }
}

fn compute_grid_cell_index(position: Point3<f32>, grid: &GridUniform) -> usize {
    let grid_origin = Vector4::from(grid.origin).truncate();
    let Point3 { x, y, z } = (position - grid_origin) / grid.offset;
    let r = grid.resolution as usize;
    (x as usize) + (y as usize * r) + (z as usize * r * r)
}

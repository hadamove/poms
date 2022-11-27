use super::molecule::{Atom, Molecule};
use cgmath::{Point3, Vector3, Vector4};

pub mod molecule_grid;
pub mod ses_grid;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GridUniform {
    origin: [f32; 4],
    // Number of grid points in each direction.
    resolution: u32,
    // The offset between each grid point.
    offset: f32,
    // Size of the grid.
    size: f32,
    // Add 4 bytes padding to avoid alignment issues.
    _padding: [u8; 4],
}

pub enum GridSpacing {
    Offset(f32),
    Resolution(u32),
}

impl GridUniform {
    pub fn from_atoms(atoms: &Vec<Atom>, spacing: GridSpacing, probe_radius: f32) -> Self {
        let max_atom_radius = atoms.get_max_atom_radius();
        let margin = 2.0 * probe_radius + max_atom_radius;

        let origin = atoms.get_min_position() - margin * Vector3::from((1.0, 1.0, 1.0));
        let size = atoms.get_max_distance() + 2.0 * margin;

        let (resolution, offset) = Self::compute_resolution_and_offset(spacing, size);

        Self {
            origin: origin.to_homogeneous().into(),
            resolution,
            offset,
            size,
            _padding: Default::default(),
        }
    }

    fn compute_resolution_and_offset(spacing: GridSpacing, size: f32) -> (u32, f32) {
        match spacing {
            GridSpacing::Offset(offset) => ((size / offset).ceil() as u32, offset),
            GridSpacing::Resolution(resolution) => (resolution, size / resolution as f32),
        }
    }
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GridCell {
    first_atom_index: u32,
    atoms_count: u32,
}

// Data structure for efficient lookup of neighboring atoms.
#[derive(Debug, Default)]
pub struct GriddedMolecule {
    pub atoms_sorted: Vec<Atom>,
    pub neighbor_grid: GridUniform,
    pub grid_cells: Vec<GridCell>,
}

impl GriddedMolecule {
    pub fn from_atoms(atoms: Vec<Atom>, probe_radius: f32) -> Self {
        let max_atom_radius = atoms.get_max_atom_radius();
        let grid_offset = probe_radius + max_atom_radius;

        let neighbor_grid =
            GridUniform::from_atoms(&atoms, GridSpacing::Offset(grid_offset), probe_radius);

        // Divide atoms into grid cells for constant look up.
        let grid_cell_indices = atoms
            .iter()
            .map(|atom| Self::compute_grid_cell_index(atom.get_position(), &neighbor_grid))
            .collect::<Vec<_>>();

        // Sort the atoms by cell index.
        let permutation = permutation::sort(&grid_cell_indices);
        let atoms_sorted = permutation.apply_slice(&atoms);
        let grid_cell_indices = permutation.apply_slice(&grid_cell_indices);

        let grid_cell_count = u32::pow(neighbor_grid.resolution, 3) as usize;
        let mut grid_cells = vec![GridCell::default(); grid_cell_count];

        // Compute grid cell start indices and size in the atoms vector
        for (atom_index, &cell_index) in grid_cell_indices.iter().enumerate() {
            if grid_cells[cell_index].atoms_count == 0 {
                grid_cells[cell_index].first_atom_index = atom_index as u32;
            }
            grid_cells[cell_index].atoms_count += 1;
        }

        Self {
            neighbor_grid,
            atoms_sorted,
            grid_cells,
        }
    }

    fn compute_grid_cell_index(position: Point3<f32>, grid: &GridUniform) -> usize {
        let grid_origin = Vector4::from(grid.origin).truncate();
        let Point3 { x, y, z } = (position - grid_origin) / grid.offset;
        let r = grid.resolution as usize;
        (x as usize) + (y as usize * r) + (z as usize * r * r)
    }
}

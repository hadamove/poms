use cgmath::Vector3;

use crate::utils::molecule::{Atom, Molecule};

const PROBE_RADIUS: f32 = 1.2;
const MAX_ATOM_RADIUS: f32 = 1.5;

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
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
    fn compute_resolution_and_offset(spacing: GridSpacing, size: f32) -> (u32, f32) {
        match spacing {
            GridSpacing::Offset(offset) => ((size / offset).ceil() as u32, offset),
            GridSpacing::Resolution(resolution) => (resolution, size / resolution as f32),
        }
    }

    fn from_molecule(molecule: &Molecule, spacing: GridSpacing) -> Self {
        let margin = 2.0 * PROBE_RADIUS + MAX_ATOM_RADIUS;
        let origin = molecule.get_min_position() - margin * Vector3::from((1.0, 1.0, 1.0));
        let size = molecule.get_max_distance() + 2.0 * margin;

        let (resolution, offset) = Self::compute_resolution_and_offset(spacing, size);

        Self {
            origin: origin.to_homogeneous().into(),
            resolution,
            offset,
            size,
            _padding: Default::default(),
        }
    }

    pub fn update_spacing(&mut self, new_spacing: GridSpacing) {
        let (resolution, offset) = Self::compute_resolution_and_offset(new_spacing, self.size);
        self.resolution = resolution;
        self.offset = offset;
    }
}

pub struct SESGrid {
    pub uniform: GridUniform,
}

impl SESGrid {
    pub fn from_molecule(molecule: &Molecule, resolution: u32) -> Self {
        Self {
            uniform: GridUniform::from_molecule(molecule, GridSpacing::Resolution(resolution)),
        }
    }

    pub fn get_resolution(&self) -> u32 {
        self.uniform.resolution
    }

    pub fn get_num_grid_points(&self) -> u32 {
        u32::pow(self.uniform.resolution, 3)
    }
}

fn compute_grid_cell_index(position: [f32; 3], grid: &GridUniform) -> usize {
    let res = grid.resolution as usize;
    let x = ((position[0] - grid.origin[0]) / grid.offset).floor() as usize;
    let y = ((position[1] - grid.origin[1]) / grid.offset).floor() as usize;
    let z = ((position[2] - grid.origin[2]) / grid.offset).floor() as usize;
    x + y * res + z * res * res
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GridCell {
    first_atom_index: u32,
    atoms_count: u32,
}

// Structure used for constant look up of atoms.
pub struct NeighborAtomGrid {
    // Origin, resolution, and offset of the grid.
    pub uniform: GridUniform,
    // Atoms sorted by grid cell index.
    pub atoms_sorted_by_grid_cells: Vec<Atom>,
    // Grid cells holding index of the first atom and atoms count.
    pub grid_cells: Vec<GridCell>,
}

impl NeighborAtomGrid {
    pub fn from_molecule(molecule: &Molecule) -> Self {
        const NEIGHBOR_ATOM_GRID_OFFSET: f32 = PROBE_RADIUS + MAX_ATOM_RADIUS;

        let uniform =
            GridUniform::from_molecule(molecule, GridSpacing::Offset(NEIGHBOR_ATOM_GRID_OFFSET));

        // Divide atoms into grid cells for constant look up.
        let mut atoms_with_cell_indices = molecule
            .atoms
            .clone()
            .into_iter()
            .map(|atom| {
                let cell_index = compute_grid_cell_index(atom.position, &uniform);
                (atom, cell_index)
            })
            .collect::<Vec<_>>();

        // Sort the atoms by cell index.
        atoms_with_cell_indices.sort_by_key(|(_, cell_index)| *cell_index);

        // Compute grid cell start indices in the atoms vector
        let grid_cell_count = u32::pow(uniform.resolution, 3) as usize;
        let mut grid_cells = vec![GridCell::default(); grid_cell_count];

        for (atom_index, &(_, cell_index)) in atoms_with_cell_indices.iter().enumerate() {
            if grid_cells[cell_index].first_atom_index == 0 {
                grid_cells[cell_index].first_atom_index = atom_index as u32;
            }
            grid_cells[cell_index].atoms_count += 1;
        }

        // Remove the cell index from the atoms.
        let atoms_sorted_by_grid_cells = atoms_with_cell_indices
            .into_iter()
            .map(|(atom, _)| atom)
            .collect::<Vec<Atom>>();

        Self {
            uniform,
            atoms_sorted_by_grid_cells,
            grid_cells,
        }
    }
}

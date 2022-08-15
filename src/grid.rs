use cgmath::Vector3;

use crate::parser::Molecule;

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
    // Add 8 bytes padding to avoid alignment issues.
    _padding: [u8; 8],
}

enum GridSpacing {
    Offset(f32),
    Resolution(u32),
}

impl GridUniform {
    fn from_molecule(molecule: &Molecule, spacing: GridSpacing) -> Self {
        let margin = 2.0 * PROBE_RADIUS + MAX_ATOM_RADIUS;
        let origin = molecule.get_min_position() - margin * Vector3::from((1.0, 1.0, 1.0));
        let size = molecule.get_max_distance() + 2.0 * margin;

        let (resolution, offset) = match spacing {
            GridSpacing::Resolution(resolution) => (resolution, size / resolution as f32),
            GridSpacing::Offset(offset) => ((size / offset).ceil() as u32, offset),
        };

        Self {
            origin: origin.to_homogeneous().into(),
            resolution,
            offset,
            _padding: [0; 8],
        }
    }
}

pub struct SESGrid {
    pub uniform: GridUniform,
}

impl SESGrid {
    pub fn from_molecule(molecule: &Molecule) -> Self {
        const DEFAULT_SES_GRID_RESOLUTION: u32 = 64;
        Self {
            uniform: GridUniform::from_molecule(
                molecule,
                GridSpacing::Resolution(DEFAULT_SES_GRID_RESOLUTION),
            ),
        }
    }
}

pub struct NeighborAtomGrid {
    uniform: GridUniform,
}

impl NeighborAtomGrid {
    pub fn from_molecule(molecule: &Molecule) -> Self {
        const NEIGHBOR_ATOM_GRID_OFFSET: f32 = PROBE_RADIUS + MAX_ATOM_RADIUS;
        Self {
            uniform: GridUniform::from_molecule(
                molecule,
                GridSpacing::Offset(NEIGHBOR_ATOM_GRID_OFFSET),
            ),
        }
    }
}

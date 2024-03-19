use cgmath::{Point3, Vector3, Vector4};

use super::atom::*;

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

pub fn create_compute_grid_around_molecule(
    atoms: &[Atom],
    resolution: u32,
    probe_radius: f32,
) -> GridUniform {
    let max_atom_radius = get_max_atom_radius(atoms);
    let margin = 2.0 * probe_radius + max_atom_radius;

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

pub fn create_atoms_lookup_grid_around_molecule(atoms: &[Atom], probe_radius: f32) -> GridUniform {
    let max_atom_radius = get_max_atom_radius(atoms);
    let margin = 2.0 * probe_radius + max_atom_radius;

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

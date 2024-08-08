use cgmath::{Point3, Vector3, Vector4};

use super::atom::*;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GridUniform {
    /// The point in space where the grid starts (the minimum x, y, z coordinates).
    pub origin: [f32; 4],
    /// Number of grid points in each direction.
    pub resolution: u32,
    /// Distance between two adjacent grid points.
    pub spacing: f32,
    /// Probe radius associated with the grid.
    pub probe_radius: f32,
    // Add 4 bytes padding to avoid alignment issues.
    _padding: [u8; 4],
}

impl GridUniform {
    pub fn change_resolution(&mut self, new_resolution: u32) {
        let size = self.resolution as f32 * self.spacing;
        self.resolution = new_resolution;
        self.spacing = size / new_resolution as f32;
    }
}

pub fn create_compute_grid_around_molecule(
    atoms: &[Atom],
    resolution: u32,
    probe_radius: f32,
) -> GridUniform {
    let max_atom_radius = get_max_atom_radius(atoms);
    let margin = 2.0 * probe_radius + max_atom_radius;

    let origin = get_min_position(atoms) - margin * Vector3::from((1.0, 1.0, 1.0));
    let span = get_max_distance(atoms) + 2.0 * margin;
    let spacing = span / resolution as f32;

    GridUniform {
        origin: origin.to_homogeneous().into(),
        resolution,
        spacing,
        probe_radius,
        _padding: Default::default(),
    }
}

pub fn create_atoms_lookup_grid_around_molecule(atoms: &[Atom], probe_radius: f32) -> GridUniform {
    let max_atom_radius = get_max_atom_radius(atoms);
    let margin = 2.0 * probe_radius + max_atom_radius;

    let origin = get_min_position(atoms) - margin * Vector3::from((1.0, 1.0, 1.0));
    let span = get_max_distance(atoms) + 2.0 * margin;
    let spacing = probe_radius + max_atom_radius;

    let resolution = (span / spacing).ceil() as u32;

    GridUniform {
        origin: origin.to_homogeneous().into(),
        spacing,
        resolution,
        probe_radius,
        _padding: Default::default(),
    }
}

pub fn position_to_voxel_index(position: Point3<f32>, grid: &GridUniform) -> usize {
    let grid_origin = Vector4::from(grid.origin).truncate();
    let Point3 { x, y, z } = (position - grid_origin) / grid.spacing;
    let r = grid.resolution as usize;
    (x as usize) + (y as usize * r) + (z as usize * r * r)
}

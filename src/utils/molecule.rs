use cgmath::Point3;

use crate::compute::grid::NeighborAtomGrid;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Atom {
    pub position: [f32; 3],
    pub radius: f32,
    pub color: [f32; 4],
}

impl Default for Atom {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            radius: 1.0,
            color: [1.0; 4],
        }
    }
}

pub struct Molecule {
    pub atoms: Vec<Atom>,
}

pub struct ComputedMolecule {
    pub mol: Molecule,
    pub neighbor_atom_grid: NeighborAtomGrid,
}

impl ComputedMolecule {
    pub fn new(mol: Molecule) -> Self {
        let neighbor_atom_grid = NeighborAtomGrid::from_molecule(&mol);
        Self {
            mol,
            neighbor_atom_grid,
        }
    }
}

impl Molecule {
    pub fn calculate_centre(&self) -> [f32; 3] {
        let mut centre = [0.0, 0.0, 0.0];
        for atom in &self.atoms {
            for (i, position) in atom.position.iter().enumerate() {
                centre[i] += position;
            }
        }
        centre.map(|v| v / self.atoms.len() as f32)
    }

    pub fn get_max_position(&self) -> Point3<f32> {
        let mut max_position = Point3::new(1.0, 1.0, 1.0) * f32::MIN;
        for atom in &self.atoms {
            for i in 0..3 {
                if atom.position[i] > max_position[i] {
                    max_position[i] = atom.position[i];
                }
            }
        }
        max_position
    }

    pub fn get_min_position(&self) -> Point3<f32> {
        let mut min_position = Point3::new(1.0, 1.0, 1.0) * f32::MAX;
        for atom in &self.atoms {
            for i in 0..3 {
                if atom.position[i] < min_position[i] {
                    min_position[i] = atom.position[i];
                }
            }
        }
        min_position
    }

    pub fn get_max_distance(&self) -> f32 {
        let min = self.get_min_position();
        let max = self.get_max_position();
        f32::max(max.x - min.x, f32::max(max.y - min.y, max.z - min.z))
    }
}

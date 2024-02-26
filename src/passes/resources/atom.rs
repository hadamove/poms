use crate::parser::parse::ParsedAtom;
use cgmath::{Bounded, Point3, Vector3};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Atom {
    pub position: [f32; 3],
    pub radius: f32,
    pub color: [f32; 4],
}

impl From<ParsedAtom> for Atom {
    fn from(atom: ParsedAtom) -> Self {
        let color = atom.element_data.jmol_color;
        Self {
            position: [
                atom.position.0 as f32,
                atom.position.1 as f32,
                atom.position.2 as f32,
            ],
            radius: atom.element_data.vdw_radius,
            color: [color[0], color[1], color[2], 1.0],
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

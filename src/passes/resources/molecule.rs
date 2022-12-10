use crate::parser::parse::ParsedAtom;
use cgmath::{Bounded, Point3, Vector3};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Atom {
    position: [f32; 3],
    radius: f32,
    color: [f32; 4],
}

impl Atom {
    pub fn get_position(&self) -> Point3<f32> {
        Point3::from(self.position)
    }
}

impl From<ParsedAtom> for Atom {
    fn from(atom: ParsedAtom) -> Self {
        let color = atom.element_data.jmol_color;
        Self {
            position: atom.position,
            radius: atom.element_data.vdw_radius,
            color: [color[0], color[1], color[2], 1.0],
        }
    }
}

pub trait Molecule {
    fn calculate_center(&self) -> Point3<f32>;
    fn get_max_distance(&self) -> f32;
    fn get_max_atom_radius(&self) -> f32;
    fn get_max_position(&self) -> Point3<f32>;
    fn get_min_position(&self) -> Point3<f32>;
}

impl Molecule for Vec<Atom> {
    fn calculate_center(&self) -> Point3<f32> {
        let mut center = Point3::new(0.0, 0.0, 0.0);
        for atom in self.iter() {
            center += Vector3::from(atom.position);
        }
        center / self.len() as f32
    }

    fn get_max_distance(&self) -> f32 {
        let min = self.get_min_position();
        let max = self.get_max_position();
        f32::max(max.x - min.x, f32::max(max.y - min.y, max.z - min.z))
    }

    fn get_max_atom_radius(&self) -> f32 {
        self.iter().map(|a| a.radius).fold(0.0, f32::max)
    }

    fn get_max_position(&self) -> Point3<f32> {
        self.iter().fold(Point3::min_value(), |res, atom| {
            let position = Point3::from(atom.position);
            Point3::new(
                f32::max(position.x, res.x),
                f32::max(position.y, res.y),
                f32::max(position.z, res.z),
            )
        })
    }

    fn get_min_position(&self) -> Point3<f32> {
        self.iter().fold(Point3::max_value(), |res, atom| {
            let position = Point3::from(atom.position);
            Point3::new(
                f32::min(position.x, res.x),
                f32::min(position.y, res.y),
                f32::min(position.z, res.z),
            )
        })
    }
}

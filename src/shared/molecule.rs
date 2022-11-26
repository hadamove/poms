use cgmath::{Bounded, Point3, Vector3};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Atom {
    pub position: [f32; 3],
    pub radius: f32,
    pub color: [f32; 4],
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
        let mut centre = Point3::new(0.0, 0.0, 0.0);
        for atom in self.iter() {
            centre += Vector3::from(atom.position);
        }
        centre / self.len() as f32
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
                position.x.max(res.x),
                position.y.max(res.y),
                position.z.max(res.z),
            )
        })
    }

    fn get_min_position(&self) -> Point3<f32> {
        self.iter().fold(Point3::max_value(), |res, atom| {
            let position = Point3::from(atom.position);
            Point3::new(
                position.x.min(res.x),
                position.y.min(res.y),
                position.z.min(res.z),
            )
        })
    }
}

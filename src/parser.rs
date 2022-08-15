use cgmath::Point3;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Atom {
    position: [f32; 3],
    radius: f32,
    color: [f32; 4],
}

pub struct Molecule {
    pub atoms: Vec<Atom>,
}

// TODO: store color pallete in a separate file
fn get_atom_color(atom_type: &str) -> [f32; 4] {
    match atom_type {
        "H" => [1.0, 1.0, 1.0, 1.0],
        "C" => [0.2, 0.2, 0.2, 1.0],
        "O" => [1.0, 0.0, 0.0, 1.0],
        "N" => [0.0, 0.0, 1.0, 1.0],
        "S" => [1.0, 1.0, 0.0, 1.0],
        "P" => [1.0, 0.5, 0.0, 1.0],
        _ => [0.0, 0.0, 0.0, 1.0],
    }
}

impl Molecule {
    pub fn calculate_centre(&self) -> [f32; 3] {
        let mut centre = [0.0, 0.0, 0.0];
        for atom in &self.atoms {
            for i in 0..3 {
                centre[i] += atom.position[i];
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
        let min_position = self.get_min_position();
        let max_position = self.get_max_position();
        return f32::max(
            max_position.x - min_position.x,
            f32::max(
                max_position.y - min_position.y,
                max_position.z - min_position.z,
            ),
        );
    }
}

pub fn load_pdb_file(filename: &String) -> String {
    #[cfg(not(target_arch = "wasm32"))]
    return std::fs::read_to_string(filename).expect("file could not be read");

    #[cfg(target_arch = "wasm32")]
    // TODO: make this function async to work with wasm
    // https://github.com/dabreegster/minimal_websys_winit_glow_demo
    crate::web_utils::fetch_file(filename).await
}

pub fn parse_pdb_file(filename: &String) -> Molecule {
    let mut atoms: Vec<Atom> = vec![];
    let content = load_pdb_file(filename);

    for line in content.split('\n') {
        if line.len() < 80 {
            continue;
        }
        if &line[0..4] == "ATOM" {
            let element = &line[77..78];
            atoms.push(Atom {
                position: [
                    line[30..38].trim().parse::<f32>().unwrap(),
                    line[38..45].trim().parse::<f32>().unwrap(),
                    line[46..53].trim().parse::<f32>().unwrap(),
                ],
                radius: 2.0,
                color: get_atom_color(element),
            });
        }
    }

    Molecule { atoms }
}

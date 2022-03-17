use std::{
    fs::File,
    io::{BufRead, BufReader},
};

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
}

pub fn parse_pdb_file(filename: String) -> Molecule {
    let mut atoms: Vec<Atom> = vec![];
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();

        if &line[0..4] == "ATOM" {
            let element = &line[77..78];
            atoms.push(Atom {
                position: [
                    line[30..38].trim().parse::<f32>().unwrap(),
                    line[38..45].trim().parse::<f32>().unwrap(),
                    line[46..53].trim().parse::<f32>().unwrap(),
                ],
                radius: 0.7,
                color: get_atom_color(element),
            });
        }
    }

    Molecule { atoms }
}

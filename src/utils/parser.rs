use super::molecule::{Atom, Molecule};

pub fn load_pdb_file(filename: &String) -> String {
    #[cfg(not(target_arch = "wasm32"))]
    return std::fs::read_to_string(filename).expect("file could not be read");

    #[cfg(target_arch = "wasm32")]
    // TODO: make this function async to work with wasm
    // https://github.com/dabreegster/minimal_websys_winit_glow_demo
    crate::wasm::fetch_file(filename).await
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
                color: get_default_color(element),
            });
        }
    }

    Molecule { atoms }
}

fn get_default_color(atom_type: &str) -> [f32; 4] {
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

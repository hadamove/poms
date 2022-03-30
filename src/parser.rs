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

pub async fn load_pdb_file(filename: String) -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read_to_string(filename).expect("file could not be read")
    }

    #[cfg(target_arch = "wasm32")]
    {
        // Since it is more difficult to access the filesystem in the browser,
        // we will fetch the file from local storage using http protocol and open port
        // Configure the port if needed based on server running the index.html file
        let url = format!("http://localhost:8000/{}", filename);
        let resp = reqwest_wasm::get(&url).await.expect("request failed");
        resp.text().await.expect("failed to read body")
    }
}

pub async fn parse_pdb_file(filename: String) -> Molecule {
    let mut atoms: Vec<Atom> = vec![];
    let content = load_pdb_file(filename).await;

    for line in content.split("\n") {
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
                radius: 0.7,
                color: get_atom_color(element),
            });
        }
    }

    Molecule { atoms }
}

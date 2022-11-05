use anyhow::{bail, Result};
use std::path::PathBuf;

use super::{
    elements,
    molecule::{Atom, Molecule},
};

pub fn load_pdb_file(filename: &PathBuf) -> Result<String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Read the file using OS file system
        return Ok(std::fs::read_to_string(filename)?);
    }

    #[cfg(target_arch = "wasm32")]
    {
        // TODO: make this function async to work with wasm
        // https://github.com/dabreegster/minimal_websys_winit_glow_demo
        crate::wasm::fetch_file(filename).await
    }
}

pub fn parse_pdb_file(filename: &PathBuf) -> Result<Molecule> {
    if !filename.to_string_lossy().ends_with(".pdb") {
        bail!("File does not end with .pdb");
    }

    let mut atoms: Vec<Atom> = vec![];
    let content = load_pdb_file(filename)?;

    for line in content.split('\n') {
        if line.len() < 78 {
            continue;
        }
        if &line[0..4] == "ATOM" {
            let symbol = &line[77..78];
            let element_data = elements::get_element_data(symbol);

            atoms.push(Atom {
                position: [
                    line[30..38].trim().parse::<f32>()?,
                    line[38..45].trim().parse::<f32>()?,
                    line[46..53].trim().parse::<f32>()?,
                ],
                radius: element_data.radius,
                color: element_data.color,
            });
        }
    }

    if atoms.is_empty() {
        bail!("Invalid pdb file.")
    }

    Ok(Molecule { atoms })
}

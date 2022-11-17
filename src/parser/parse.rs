use anyhow::{bail, Result};

use super::elements;
use crate::shared::molecule::Atom;

pub fn parse_atoms_from_pdb_file(file: &[u8]) -> Result<Vec<Atom>> {
    let mut atoms: Vec<Atom> = vec![];
    let content = std::str::from_utf8(file)?;

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

    Ok(atoms)
}

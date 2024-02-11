use std::io::{BufReader, Cursor};

use anyhow::{bail, Result};

use super::elements::{self, ElementData};

pub struct ParsedAtom {
    pub position: (f64, f64, f64),
    pub element_data: ElementData,
}

pub type ParsedFile = Vec<ParsedAtom>;

fn format_errors(errors: Vec<pdbtbx::PDBError>) -> String {
    errors
        .into_iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_atoms_from_pdb_file(file: &[u8]) -> Result<ParsedFile> {
    let buff = BufReader::new(Cursor::new(file));
    let result = pdbtbx::open_raw(buff, pdbtbx::StrictnessLevel::Loose);

    let pdb = match result {
        Err(e) => bail!("❌ Something went wrong with parsing: {}", format_errors(e)),
        Ok((pdb, e)) => {
            if !e.is_empty() {
                println!("❗️ Errors during parsing: {}", format_errors(e));
            }
            pdb
        }
    };

    let atoms = pdb
        .atoms()
        .map(|atom| ParsedAtom {
            position: atom.pos(),
            element_data: elements::get_element_data(atom.element().unwrap_or(&pdbtbx::Element::H)),
        })
        .collect::<Vec<_>>();

    Ok(atoms)
}

pub fn parse_files(files: Vec<Vec<u8>>) -> Result<Vec<ParsedFile>> {
    files
        .iter()
        .map(|file| parse_atoms_from_pdb_file(file))
        .collect::<anyhow::Result<Vec<_>>>()
}

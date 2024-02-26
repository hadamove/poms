use std::io::{BufReader, Cursor};

use anyhow::{bail, Result};

use super::elements::{self, ElementData};

pub struct ParsedAtom {
    pub position: (f64, f64, f64),
    pub element_data: ElementData,
}

pub struct ParsedMolecule {
    /// The identifier as posed in the PDB Header or mmCIF entry.id
    pub header: Option<String>,
    /// The atoms parsed from the file
    pub atoms: Vec<ParsedAtom>,
}

// TODO: please refactor this
fn format_errors(errors: Vec<pdbtbx::PDBError>) -> String {
    errors
        .into_iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

// TODO: please refactor this
fn parse_atoms_from_pdb_file(file: &[u8]) -> Result<ParsedMolecule> {
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

    Ok(ParsedMolecule {
        header: pdb.identifier,
        atoms,
    })
}

pub fn parse_files(files: Vec<Vec<u8>>) -> Result<Vec<ParsedMolecule>> {
    files
        .iter()
        .map(|file| parse_atoms_from_pdb_file(file))
        .collect::<anyhow::Result<Vec<_>>>()
}

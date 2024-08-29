use std::io::{BufReader, Cursor};

use poms_common::limits::MAX_NUM_ATOMS;
use poms_common::models::atom::Atom;

use super::file_loader::RawFile;

/// A parsed molecule from a PDB or mmCIF file.
pub struct ParsedMolecule {
    pub filename: String,
    /// The identifier as posed in the PDB Header or mmCIF entry.id.
    pub header: Option<String>,
    /// The atoms parsed from the file.
    pub atoms: Vec<Atom>,
}

/// Attempts to parse a PDB or mmCIF file as bytes into a [`ParsedMolecule`].
pub fn parse_atoms_from_pdb_file(file: RawFile) -> anyhow::Result<ParsedMolecule> {
    let buffer = BufReader::new(Cursor::new(&file.content));
    let (pdb, _) = pdbtbx::open_raw(buffer, pdbtbx::StrictnessLevel::Loose)
        .map_err(|errors| anyhow::Error::msg(format_parse_errors(&errors)))?;

    let atoms = pdb
        .atoms()
        .map(|atom| Atom {
            position: {
                let (x, y, z) = atom.pos();
                [x as f32, y as f32, z as f32]
            },
            radius: get_vdw_radius(atom),
            color: get_jmol_color(atom),
        })
        .collect::<Vec<_>>();

    if atoms.len() > MAX_NUM_ATOMS {
        return Err(anyhow::Error::msg(format!(
            "Number of atoms in the file exceeds the limit ({}).",
            MAX_NUM_ATOMS
        )));
    }

    Ok(ParsedMolecule {
        filename: file.name,
        header: pdb.identifier,
        atoms,
    })
}

fn format_parse_errors(errors: &[pdbtbx::PDBError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

fn get_vdw_radius(atom: &pdbtbx::Atom) -> f32 {
    atom.element()
        .and_then(|e| e.atomic_radius().van_der_waals)
        .unwrap_or(1.0) as f32
}

fn get_jmol_color(atom: &pdbtbx::Atom) -> [f32; 4] {
    const DEFAULT_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    let Some(element) = atom.element() else {
        return DEFAULT_COLOR;
    };
    match element {
        pdbtbx::Element::H => [1.0, 1.0, 1.0, 1.0],
        pdbtbx::Element::He => [0.85, 1.0, 1.0, 1.0],
        pdbtbx::Element::Li => [0.8, 0.5, 1.0, 1.0],
        pdbtbx::Element::Be => [0.76, 1.0, 0.0, 1.0],
        pdbtbx::Element::B => [1.0, 0.71, 0.71, 1.0],
        pdbtbx::Element::C => [0.56, 0.56, 0.56, 1.0],
        pdbtbx::Element::N => [0.19, 0.31, 0.97, 1.0],
        pdbtbx::Element::O => [1.0, 0.05, 0.05, 1.0],
        pdbtbx::Element::F => [0.56, 0.88, 0.31, 1.0],
        pdbtbx::Element::Ne => [0.7, 0.89, 0.96, 1.0],
        pdbtbx::Element::Na => [0.67, 0.36, 0.95, 1.0],
        pdbtbx::Element::Mg => [0.54, 1.0, 0.0, 1.0],
        pdbtbx::Element::Al => [0.75, 0.65, 0.65, 1.0],
        pdbtbx::Element::Si => [0.94, 0.78, 0.63, 1.0],
        pdbtbx::Element::P => [1.0, 0.5, 0.0, 1.0],
        pdbtbx::Element::S => [1.0, 1.0, 0.19, 1.0],
        pdbtbx::Element::Cl => [0.12, 0.94, 0.12, 1.0],
        pdbtbx::Element::Ar => [0.5, 0.82, 0.89, 1.0],
        pdbtbx::Element::K => [0.56, 0.25, 0.83, 1.0],
        pdbtbx::Element::Ca => [0.24, 1.0, 0.0, 1.0],
        pdbtbx::Element::Sc => [0.9, 0.9, 0.9, 1.0],
        pdbtbx::Element::Ti => [0.75, 0.76, 0.78, 1.0],
        pdbtbx::Element::V => [0.65, 0.65, 0.67, 1.0],
        pdbtbx::Element::Cr => [0.54, 0.6, 0.78, 1.0],
        pdbtbx::Element::Mn => [0.61, 0.48, 0.78, 1.0],
        pdbtbx::Element::Fe => [0.88, 0.4, 0.2, 1.0],
        pdbtbx::Element::Co => [0.94, 0.56, 0.63, 1.0],
        pdbtbx::Element::Ni => [0.31, 0.82, 0.31, 1.0],
        pdbtbx::Element::Cu => [0.78, 0.5, 0.2, 1.0],
        pdbtbx::Element::Zn => [0.49, 0.5, 0.69, 1.0],
        pdbtbx::Element::Ga => [0.76, 0.56, 0.56, 1.0],
        pdbtbx::Element::Ge => [0.4, 0.56, 0.56, 1.0],
        pdbtbx::Element::As => [0.74, 0.5, 0.89, 1.0],
        pdbtbx::Element::Se => [1.0, 0.63, 0.0, 1.0],
        pdbtbx::Element::Br => [0.65, 0.16, 0.16, 1.0],
        pdbtbx::Element::Kr => [0.36, 0.72, 0.82, 1.0],
        pdbtbx::Element::Rb => [0.44, 0.18, 0.69, 1.0],
        pdbtbx::Element::Sr => [0.0, 1.0, 0.0, 1.0],
        pdbtbx::Element::Y => [0.58, 1.0, 1.0, 1.0],
        pdbtbx::Element::Zr => [0.58, 0.88, 0.88, 1.0],
        pdbtbx::Element::Nb => [0.45, 0.76, 0.79, 1.0],
        pdbtbx::Element::Mo => [0.33, 0.71, 0.71, 1.0],
        pdbtbx::Element::Tc => [0.23, 0.62, 0.62, 1.0],
        pdbtbx::Element::Ru => [0.14, 0.56, 0.56, 1.0],
        pdbtbx::Element::Rh => [0.04, 0.49, 0.55, 1.0],
        pdbtbx::Element::Pd => [0.0, 0.41, 0.52, 1.0],
        pdbtbx::Element::Ag => [0.75, 0.75, 0.75, 1.0],
        pdbtbx::Element::Cd => [1.0, 0.85, 0.56, 1.0],
        pdbtbx::Element::In => [0.65, 0.46, 0.45, 1.0],
        pdbtbx::Element::Sn => [0.4, 0.5, 0.5, 1.0],
        pdbtbx::Element::Sb => [0.62, 0.39, 0.71, 1.0],
        pdbtbx::Element::Te => [0.83, 0.48, 0.0, 1.0],
        pdbtbx::Element::I => [0.58, 0.0, 0.58, 1.0],
        pdbtbx::Element::Xe => [0.26, 0.62, 0.69, 1.0],
        pdbtbx::Element::Cs => [0.34, 0.09, 0.56, 1.0],
        pdbtbx::Element::Ba => [0.0, 0.79, 0.0, 1.0],
        pdbtbx::Element::La => [0.44, 0.83, 1.0, 1.0],
        pdbtbx::Element::Ce => [1.0, 1.0, 0.78, 1.0],
        pdbtbx::Element::Pr => [0.85, 1.0, 0.78, 1.0],
        pdbtbx::Element::Nd => [0.78, 1.0, 0.78, 1.0],
        pdbtbx::Element::Pm => [0.64, 1.0, 0.78, 1.0],
        pdbtbx::Element::Sm => [0.56, 1.0, 0.78, 1.0],
        pdbtbx::Element::Eu => [0.38, 1.0, 0.78, 1.0],
        pdbtbx::Element::Gd => [0.27, 1.0, 0.78, 1.0],
        pdbtbx::Element::Tb => [0.19, 1.0, 0.78, 1.0],
        pdbtbx::Element::Dy => [0.12, 1.0, 0.78, 1.0],
        pdbtbx::Element::Ho => [0.0, 1.0, 0.61, 1.0],
        pdbtbx::Element::Er => [0.0, 0.9, 0.46, 1.0],
        pdbtbx::Element::Tm => [0.0, 0.83, 0.32, 1.0],
        pdbtbx::Element::Yb => [0.0, 0.75, 0.22, 1.0],
        pdbtbx::Element::Lu => [0.0, 0.67, 0.14, 1.0],
        pdbtbx::Element::Hf => [0.3, 0.76, 1.0, 1.0],
        pdbtbx::Element::Ta => [0.3, 0.65, 1.0, 1.0],
        pdbtbx::Element::W => [0.12, 0.56, 1.0, 1.0],
        pdbtbx::Element::Re => [0.15, 0.49, 0.67, 1.0],
        pdbtbx::Element::Os => [0.15, 0.4, 0.59, 1.0],
        pdbtbx::Element::Ir => [0.09, 0.33, 0.53, 1.0],
        pdbtbx::Element::Pt => [0.0, 0.31, 0.49, 1.0],
        pdbtbx::Element::Au => [1.0, 0.82, 0.12, 1.0],
        pdbtbx::Element::Hg => [0.72, 0.72, 0.82, 1.0],
        pdbtbx::Element::Tl => [0.65, 0.32, 0.3, 1.0],
        pdbtbx::Element::Pb => [0.34, 0.35, 0.38, 1.0],
        pdbtbx::Element::Bi => [0.62, 0.31, 0.71, 1.0],
        pdbtbx::Element::Po => [0.67, 0.36, 0.0, 1.0],
        pdbtbx::Element::At => [0.46, 0.31, 0.27, 1.0],
        pdbtbx::Element::Rn => [0.26, 0.51, 0.59, 1.0],
        pdbtbx::Element::Fr => [0.26, 0.0, 0.4, 1.0],
        pdbtbx::Element::Ra => [0.0, 0.49, 0.0, 1.0],
        pdbtbx::Element::Ac => [0.44, 0.67, 0.98, 1.0],
        pdbtbx::Element::Th => [0.0, 0.73, 1.0, 1.0],
        pdbtbx::Element::Pa => [0.0, 0.63, 1.0, 1.0],
        pdbtbx::Element::U => [0.0, 0.56, 1.0, 1.0],
        pdbtbx::Element::Np => [0.0, 0.5, 1.0, 1.0],
        pdbtbx::Element::Pu => [0.0, 0.42, 1.0, 1.0],
        pdbtbx::Element::Am => [0.33, 0.36, 0.95, 1.0],
        pdbtbx::Element::Cm => [0.47, 0.36, 0.89, 1.0],
        pdbtbx::Element::Bk => [0.54, 0.31, 0.89, 1.0],
        pdbtbx::Element::Cf => [0.63, 0.21, 0.83, 1.0],
        pdbtbx::Element::Es => [0.7, 0.12, 0.83, 1.0],
        pdbtbx::Element::Fm => [0.7, 0.12, 0.73, 1.0],
        pdbtbx::Element::Md => [0.7, 0.05, 0.65, 1.0],
        pdbtbx::Element::No => [0.74, 0.05, 0.53, 1.0],
        pdbtbx::Element::Lr => [0.78, 0.0, 0.41, 1.0],
        _ => DEFAULT_COLOR,
    }
}

impl ParsedMolecule {
    pub fn h2o_demo() -> Self {
        Self {
            filename: "demo".to_string(),
            header: Some("h2o".to_string()),
            atoms: vec![
                // Oxygen
                Atom {
                    position: [0.0, 0.0, 0.0],
                    radius: 1.4,
                    color: [1.0, 0.0, 0.0, 1.0],
                },
                // Hydrogen 1
                Atom {
                    position: [0.9572, 0.0, 0.0],
                    radius: 1.2,
                    color: [1.0, 1.0, 1.0, 1.0],
                },
                // Hydrogen 2
                Atom {
                    position: [-0.2396, 0.927, 0.0],
                    radius: 1.2,
                    color: [1.0, 1.0, 1.0, 1.0],
                },
            ],
        }
    }
}

use std::io::{BufReader, Cursor};

use super::RawFile;
use poms_common::limits::MAX_NUM_ATOMS;
use poms_common::models::atom::Atom;

/// A parsed molecule from a PDB or mmCIF file.
pub(crate) struct ParsedMolecule {
    pub(crate) filename: String,
    pub(crate) atoms: Vec<Atom>,
}

/// Attempts to parse a PDB or mmCIF file as bytes into a [`ParsedMolecule`].
pub(crate) fn parse_atoms_from_pdb_file(file: RawFile) -> anyhow::Result<ParsedMolecule> {
    let buffer = BufReader::new(Cursor::new(&file.content));

    let atoms = match pdbtbx::open_raw(buffer, pdbtbx::StrictnessLevel::Loose) {
        Ok((pdb, _)) => pdb.atoms().map(convert_to_internal_atom).collect(),
        // If pdbtbx fails to parse the file (e.g. due to missing header), fallback to a simple parser.
        Err(_) => simple_parser::try_parse_pdb(file.content)?,
    };

    if atoms.len() > MAX_NUM_ATOMS {
        return Err(anyhow::Error::msg(format!(
            "Number of atoms in the file exceeds the limit ({}).",
            MAX_NUM_ATOMS
        )));
    }

    Ok(ParsedMolecule {
        filename: file.name,
        atoms,
    })
}

/// A convenience function to parse multiple PDB or mmCIF files at once.
pub(crate) fn parse_multiple_files(
    loaded_files: Vec<RawFile>,
) -> anyhow::Result<Vec<ParsedMolecule>> {
    loaded_files
        .into_iter()
        .map(parse_atoms_from_pdb_file)
        .collect::<anyhow::Result<Vec<ParsedMolecule>>>()
}

macro_rules! extract_file_name {
    ($path:expr) => {{
        $path.split('/').last().unwrap()
    }};
}

/// A convenience macro to parse a single PDB or mmCIF file from path.
macro_rules! include_molecule {
    ($file_path:expr) => {{
        use crate::app::data::RawFile;
        let file_name = extract_file_name!($file_path);
        let demo_file = include_bytes!($file_path);
        let initial_molecule = data::molecule_parser::parse_atoms_from_pdb_file(RawFile {
            name: file_name.to_string(),
            content: demo_file.to_vec(),
        })
        .unwrap();
        initial_molecule
    }};
}

/// Converts `pdbtbx::Atom` to our internal `Atom` struct.
fn convert_to_internal_atom(atom: &pdbtbx::Atom) -> Atom {
    Atom {
        position: {
            let (x, y, z) = atom.pos();
            [x as f32, y as f32, z as f32]
        },
        radius: get_vdw_radius(atom.element()),
        color: get_jmol_color(atom.element()),
    }
}

fn get_vdw_radius(element: Option<&pdbtbx::Element>) -> f32 {
    const DEFAULT_RADIUS: f64 = 1.0;
    element
        .map(|e| e.atomic_radius().van_der_waals.unwrap_or(DEFAULT_RADIUS))
        .unwrap_or(DEFAULT_RADIUS) as f32
}

fn get_jmol_color(element: Option<&pdbtbx::Element>) -> [f32; 4] {
    const DEFAULT_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    let Some(element) = element else {
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

mod simple_parser {

    use super::Atom;
    use std::ops::Range;

    const MIN_LINE_LENGTH: usize = 78;
    const LINE_PREFIX: Range<usize> = 0..4;
    const LINE_POSITION_X: Range<usize> = 30..38;
    const LINE_POSITION_Y: Range<usize> = 38..46;
    const LINE_POSITION_Z: Range<usize> = 46..54;
    const LINE_ELEMENT_SYMBOL: Range<usize> = 77..78;

    pub(crate) fn try_parse_pdb(content: Vec<u8>) -> anyhow::Result<Vec<Atom>> {
        let mut atoms: Vec<Atom> = vec![];
        let content = std::str::from_utf8(&content)?;

        for line in content.split('\n') {
            if line.len() < MIN_LINE_LENGTH {
                continue;
            }
            if &line[LINE_PREFIX] == "ATOM" {
                let symbol = &line[LINE_ELEMENT_SYMBOL];
                let element = pdbtbx::Element::try_from(symbol).ok();

                atoms.push(Atom {
                    position: parse_position_from_line(line)?,
                    radius: super::get_vdw_radius(element.as_ref()),
                    color: super::get_jmol_color(element.as_ref()),
                });
            }
        }
        match atoms.len() {
            0 => anyhow::bail!("No atoms found in file"),
            _ => Ok(atoms),
        }
    }

    fn parse_position_from_line(line: &str) -> anyhow::Result<[f32; 3]> {
        Ok([
            line[LINE_POSITION_X].trim().parse::<f32>()?,
            line[LINE_POSITION_Y].trim().parse::<f32>()?,
            line[LINE_POSITION_Z].trim().parse::<f32>()?,
        ])
    }
}

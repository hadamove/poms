use std::ops::Range;

use anyhow::{bail, Result};

use super::elements::{self, ElementData};

const MIN_LINE_LENGTH: usize = 78;
const LINE_PREFIX: Range<usize> = 0..4;
const LINE_POSITION_X: Range<usize> = 30..38;
const LINE_POSITION_Y: Range<usize> = 38..46;
const LINE_POSITION_Z: Range<usize> = 46..54;
const LINE_ELEMENT_SYMBOL: Range<usize> = 77..78;

pub struct ParsedAtom {
    pub position: [f32; 3],
    pub element_data: ElementData,
}

pub type ParsedFile = Vec<ParsedAtom>;

pub fn parse_files(files: Vec<Vec<u8>>) -> Result<Vec<ParsedFile>> {
    files
        .iter()
        .map(|file| parse_atoms_from_pdb_file(file))
        .collect::<anyhow::Result<Vec<_>>>()
}

fn parse_position_from_line(line: &str) -> Result<[f32; 3]> {
    Ok([
        line[LINE_POSITION_X].trim().parse::<f32>()?,
        line[LINE_POSITION_Y].trim().parse::<f32>()?,
        line[LINE_POSITION_Z].trim().parse::<f32>()?,
    ])
}

fn parse_atoms_from_pdb_file(file: &[u8]) -> Result<ParsedFile> {
    let mut atoms: Vec<ParsedAtom> = vec![];
    let content = std::str::from_utf8(file)?;

    for line in content.split('\n') {
        if line.len() < MIN_LINE_LENGTH {
            continue;
        }
        if &line[LINE_PREFIX] == "ATOM" {
            let symbol = &line[LINE_ELEMENT_SYMBOL];

            atoms.push(ParsedAtom {
                position: parse_position_from_line(line)?,
                element_data: elements::get_element_data(symbol),
            });
        }
    }
    match atoms.len() {
        0 => bail!("No atoms found in file"),
        _ => Ok(atoms),
    }
}

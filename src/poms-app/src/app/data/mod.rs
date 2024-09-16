use std::{fmt::Display, str::FromStr};

pub(crate) mod file_loader;
#[macro_use]
pub(crate) mod molecule_parser;
pub(crate) mod molecule_storage;
mod pdb_apis;

/// Holds the raw content of a loaded file.
pub(crate) struct RawFile {
    pub(crate) name: String,
    pub(crate) content: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Assembly {
    pdb_id: String,
    assembly_id: String,
}

impl FromStr for Assembly {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid format: {}", s));
        }

        Ok(Assembly {
            pdb_id: parts[0].to_string(),
            assembly_id: parts[1].to_string(),
        })
    }
}

impl Display for Assembly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.pdb_id, self.assembly_id)
    }
}

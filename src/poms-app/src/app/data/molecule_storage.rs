use std::collections::HashMap;
use uuid::Uuid;

use super::molecule_parser::ParsedMolecule;
use poms_common::models::atom::{Atom, AtomsWithLookup};

pub struct MoleculeFrame {
    pub filename: String,
    pub header: Option<String>,
    pub atoms: AtomsWithLookup,
}

#[derive(Default)]
pub struct MoleculeIndex {
    pub id: Uuid,
    pub index: usize,
}

pub struct MoleculeStorage {
    /// Id of the molecule currently opened for viewing.
    current_index: MoleculeIndex,
    /// Molecules that are preloaded and ready to be displayed.
    loadded_molecules: HashMap<Uuid, Vec<MoleculeFrame>>,
}

impl MoleculeStorage {
    pub fn new(initial_molecule: ParsedMolecule, probe_radius: f32) -> Self {
        let mut storage = Self {
            current_index: MoleculeIndex::default(),
            loadded_molecules: HashMap::new(),
        };
        // Add the initial molecule to the storage
        storage.add_from_parsed(vec![initial_molecule], probe_radius);
        storage
    }

    /// Returns data associated with currently opened molecule.
    pub fn get_current(&self) -> &MoleculeFrame {
        self.loadded_molecules
            .get(&self.current_index.id)
            .and_then(|frames| frames.get(self.current_index.index))
            .expect("current should always be valid")
    }

    /// Adds a new molecule to the storage. The molecule is preprocessed for fast neighbor look up. Returns a reference to the molecule data.
    pub fn add_from_parsed(&mut self, parsed_molecules: Vec<ParsedMolecule>, probe_radius: f32) {
        // Generate a new unique id for the molecule
        let id = Uuid::new_v4();

        // Process each molecule frame
        for ParsedMolecule {
            filename,
            header,
            atoms,
        } in parsed_molecules
        {
            // Create data structure for efficient neighbor lookup needed for molecular surface algorithm
            let atoms =
                AtomsWithLookup::new(atoms.into_iter().map(Atom::from).collect(), probe_radius);

            let frame = MoleculeFrame {
                filename,
                header,
                atoms,
            };

            // Add the molecule to the storage
            self.loadded_molecules.entry(id).or_default().push(frame);
        }

        self.current_index = MoleculeIndex { id, index: 0 };
    }

    pub fn on_probe_radius_changed(&mut self, probe_radius: f32) {
        // In case probe radius changes, neighbor lookup has to be recomputed, as the spacing of the grid depends on it.
        for molecule_data in &mut self.loadded_molecules.values_mut() {
            // Process each frame in the molecule
            for frame in molecule_data {
                // Use `std::mem::take` to avoid reallocation of data
                let atoms_data = std::mem::take(&mut frame.atoms.data);
                frame.atoms = AtomsWithLookup::new(atoms_data, probe_radius);
            }
        }
    }

    pub fn next_frame(&mut self) {
        let frames = self.loadded_molecules.get(&self.current_index.id).unwrap();
        self.current_index.index = (self.current_index.index + 1) % frames.len();
    }
}

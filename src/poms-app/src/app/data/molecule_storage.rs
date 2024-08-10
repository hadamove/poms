use poms_common::models::atom::{Atom, AtomsWithLookup};
use uuid::Uuid;

use std::collections::HashMap;

use super::molecule_parser::ParsedMolecule;

// TODO: Add `frames` instead of `atoms` to store the data for each frame.
pub struct MoleculeData {
    /// The unique identifier of the molecule within the application. Generated after parsing the molecule from a file.
    pub id: Uuid,
    /// The identifier as posed in the PDB Header or mmCIF entry.id
    pub header: Option<String>,
    /// Atoms in the molecule with associated data structure for fast lookup.
    pub atoms: AtomsWithLookup,
}

pub struct MoleculeStorage {
    /// Id of the molecule currently opened for viewing.
    current: Uuid,
    /// Molecules that are preloaded and ready to be displayed.
    loadded_molecules: HashMap<Uuid, MoleculeData>,
}

impl MoleculeStorage {
    pub fn new(initial_molecule: ParsedMolecule, probe_radius: f32) -> Self {
        let mut storage = Self {
            current: Uuid::new_v4(),
            loadded_molecules: HashMap::new(),
        };

        // this could solve our weird problem
        storage.add_from_parsed(initial_molecule, probe_radius);
        storage
    }

    /// Returns data associated with currently opened molecule.
    pub fn get_current(&self) -> &MoleculeData {
        self.loadded_molecules
            .get(&self.current)
            .expect("current should always be valid")
    }

    /// Adds a new molecule to the storage. The molecule is preprocessed for fast neighbor look up. Returns a reference to the molecule data.
    pub fn add_from_parsed(
        &mut self,
        parsed_molecule: ParsedMolecule,
        probe_radius: f32,
    ) -> &MoleculeData {
        let ParsedMolecule { atoms, header } = parsed_molecule;

        // Convert atoms to our internal representation.
        let atoms = atoms.into_iter().map(Atom::from).collect::<Vec<_>>();

        // Create data structure for efficient neighbor lookup needed for molecular surface algorithm
        let atoms = AtomsWithLookup::new(atoms, probe_radius);

        let id = Uuid::new_v4();
        let molecule_data = MoleculeData { id, header, atoms };
        self.loadded_molecules.insert(id, molecule_data);
        self.current = id;

        self.get_current()
    }

    // TODO: Only recompute current molecule, not all of them?
    pub fn on_probe_radius_changed(&mut self, probe_radius: f32) {
        // In case probe radius changes, neighbor lookup has to be recomputed, as the spacing of the grid depends on it.
        for molecule_data in &mut self.loadded_molecules.values_mut() {
            // Avoid reallocation of atoms data
            let atoms_data = std::mem::take(&mut molecule_data.atoms.data);
            molecule_data.atoms = AtomsWithLookup::new(atoms_data, probe_radius);
        }
    }
}

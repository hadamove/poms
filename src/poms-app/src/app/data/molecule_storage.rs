use super::molecule_parser::ParsedMolecule;
use poms_common::models::atom::{Atom, AtomsWithLookup};

pub struct MoleculeData {
    pub filename: String,
    pub header: Option<String>,
    pub atoms: AtomsWithLookup,
}

pub struct MoleculeStorage {
    /// Index of the molecule currently opened for viewing.
    pub active_index: usize,
    /// Molecules that are preloaded and ready to be displayed.
    pub loaded_molecules: Vec<MoleculeData>,
}

impl MoleculeStorage {
    pub fn new(initial_molecule: ParsedMolecule, probe_radius: f32) -> Self {
        let mut storage = Self {
            active_index: 0,
            loaded_molecules: Vec::new(),
        };
        // Add the initial molecule to the storage
        storage.add_from_parsed(vec![initial_molecule], probe_radius);
        storage
    }

    /// Returns data associated with currently opened molecule.
    pub fn get_active(&self) -> &MoleculeData {
        self.loaded_molecules
            .get(self.active_index)
            .expect("`active_index` should always be valid")
    }

    /// Sets the molecule at the given index as active.
    pub fn set_active(&mut self, index: usize) {
        if index < self.loaded_molecules.len() {
            self.active_index = index;
        }
    }

    /// Increments the active molecule index. If the index reaches the end of the list, it wraps around.
    pub fn increment_active(&mut self) {
        self.active_index = (self.active_index + 1) % self.loaded_molecules.len();
    }

    /// Adds a new molecule to the storage. The molecule is preprocessed for fast neighbor look up. Returns a reference to the molecule data.
    pub fn add_from_parsed(&mut self, parsed_molecules: Vec<ParsedMolecule>, probe_radius: f32) {
        if parsed_molecules.is_empty() {
            return;
        }

        // Set the first added molecule as active
        self.active_index = self.loaded_molecules.len();

        for ParsedMolecule {
            filename,
            header,
            atoms,
        } in parsed_molecules
        {
            // Create data structure for efficient neighbor lookup needed for molecular surface algorithm
            let atoms =
                AtomsWithLookup::new(atoms.into_iter().map(Atom::from).collect(), probe_radius);

            let molecule_data = MoleculeData {
                filename,
                header,
                atoms,
            };

            // Add the molecule to the storage
            self.loaded_molecules.push(molecule_data);
        }
    }

    pub fn on_probe_radius_changed(&mut self, probe_radius: f32) {
        // In case probe radius changes, neighbor lookup has to be recomputed, as the spacing of the grid depends on it.
        for molecule in &mut self.loaded_molecules {
            // Use `std::mem::take` to avoid reallocation of data
            let atoms_data = std::mem::take(&mut molecule.atoms.data);
            molecule.atoms = AtomsWithLookup::new(atoms_data, probe_radius);
        }
    }
}

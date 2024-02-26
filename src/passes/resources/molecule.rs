use uuid::Uuid;

use super::atom::Atom;
use super::grid::AtomsWithLookup;
use crate::parser::ParsedMolecule;

use std::collections::HashMap;

pub struct MoleculeData {
    /// The unique identifier of the molecule within the application. Generated after parsing the molecule from a file.
    pub id: Uuid,
    /// The identifier as posed in the PDB Header or mmCIF entry.id
    pub header: Option<String>,
    /// Atoms in the molecule, for fast lookup ... TODO
    pub atoms: AtomsWithLookup,
}

#[derive(Default)]
pub struct MoleculeStorage {
    /// Id of the molecule currently opened for viewing.
    current: Option<Uuid>,
    /// Molecules that are preloaded and ready to be displayed.
    loadded_molecules: HashMap<Uuid, MoleculeData>,
}

impl MoleculeStorage {
    /// Creates a new molecule storage.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a molecule by its unique identifier.
    pub fn get_by_id(&self, id: Uuid) -> Option<&MoleculeData> {
        self.loadded_molecules.get(&id)
    }

    /// Returns data associated with currently opened molecule.
    pub fn get_current(&self) -> Option<&MoleculeData> {
        self.current.and_then(|id| self.get_by_id(id))
    }

    /// Adds a new molecule to the storage. The molecule is preprocessed for fast neighbor look up.
    pub fn add_from_parsed(&mut self, parsed_molecule: ParsedMolecule, probe_radius: f32) {
        let ParsedMolecule { atoms, header } = parsed_molecule;

        // Convert atoms to our internal representation.
        let atoms = atoms.into_iter().map(Atom::from).collect::<Vec<_>>();

        // Create data structure for efficient neighbor lookup necessary for molecular surface algorithm
        let atoms = AtomsWithLookup::new(atoms, probe_radius);

        let id = Uuid::new_v4();
        let molecule_data = MoleculeData { id, header, atoms };
        self.loadded_molecules.insert(id, molecule_data);
        self.current = Some(id);
    }

    pub fn on_probe_radius_changed(&mut self, probe_radius: f32) {
        // In case probe radius changes, neighbor lookup has to be recomputed, as the spacing of the grid depends on it.
        for molecule_data in &mut self.loadded_molecules.values_mut() {
            // Avoid reallocation of atoms data
            let atoms_data = std::mem::take(&mut molecule_data.atoms.data);
            molecule_data.atoms = AtomsWithLookup::new(atoms_data, probe_radius);
        }
    }
}

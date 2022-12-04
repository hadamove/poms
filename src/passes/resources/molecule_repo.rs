use super::{grid::GriddedMolecule, molecule::Atom};
use crate::parser::parse::ParsedFile;
use std::sync::Arc;

#[derive(Default)]
pub struct MoleculeRepo {
    molecules: Vec<Arc<GriddedMolecule>>,
    current_molecule_index: usize,
    update_molecule: bool,
    animation_frame: usize,
}

impl MoleculeRepo {
    pub fn get_new(&mut self) -> Option<Arc<GriddedMolecule>> {
        match self.update_molecule {
            true => {
                self.update_molecule = false;
                self.molecules.get(self.current_molecule_index).cloned()
            }
            false => None,
        }
    }

    pub fn get_current(&self) -> Option<Arc<GriddedMolecule>> {
        self.molecules.get(self.current_molecule_index).cloned()
    }

    pub fn increase_frame(&mut self) {
        if self.molecules.len() > 1 {
            self.current_molecule_index = self.animation_frame % self.molecules.len();
            self.update_molecule = true;
            self.animation_frame += 1;
        }
    }

    pub fn load_from_parsed(&mut self, molecules: Vec<ParsedFile>, probe_radius: f32) {
        let molecules = molecules.into_iter().map(|molecule| {
            molecule
                .into_iter()
                .map(|atom| atom.into())
                .collect::<Vec<Atom>>()
        });

        self.molecules = self.compute_neighbor_grids(molecules, probe_radius);
        self.current_molecule_index = 0;
        self.update_molecule = true;
    }

    pub fn recompute_neighbor_grids(&mut self, probe_radius: f32) {
        // The collect is necessary to avoid double borrowing of self.
        #[allow(clippy::needless_collect)]
        let atoms_cloned = self
            .molecules
            .iter()
            .map(|molecule| molecule.atoms_sorted.clone())
            .collect::<Vec<_>>();

        self.molecules = self.compute_neighbor_grids(atoms_cloned.into_iter(), probe_radius);
        self.update_molecule = true;
    }

    fn compute_neighbor_grids(
        &mut self,
        molecules: impl Iterator<Item = Vec<Atom>>,
        probe_radius: f32,
    ) -> Vec<Arc<GriddedMolecule>> {
        molecules
            .map(|molecule| Arc::new(GriddedMolecule::from_atoms(molecule, probe_radius)))
            .collect()
    }
}

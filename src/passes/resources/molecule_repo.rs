use super::molecule::Molecule;
use super::{grid::MoleculeWithNeighborGrid, molecule::Atom};
use crate::parser::parse::ParsedFile;
use crate::utils::constants::{ANIMATION_ACTIVE_BY_DEFAULT, DEFAULT_ANIMATION_SPEED};
use std::sync::Arc;

pub struct MoleculeRepo {
    molecules: Vec<Arc<MoleculeWithNeighborGrid>>,
    current_molecule_index: usize,
    update_molecule: bool,

    animation_frame: usize,
    is_animation_active: bool,
    animation_speed: u32,
}

impl Default for MoleculeRepo {
    fn default() -> Self {
        Self {
            molecules: Vec::new(),
            current_molecule_index: 0,
            update_molecule: false,

            animation_frame: 0,
            is_animation_active: ANIMATION_ACTIVE_BY_DEFAULT,
            animation_speed: DEFAULT_ANIMATION_SPEED,
        }
    }
}

impl MoleculeRepo {
    pub fn get_new(&mut self) -> Option<Arc<MoleculeWithNeighborGrid>> {
        match self.update_molecule {
            true => {
                self.update_molecule = false;
                self.molecules.get(self.current_molecule_index).cloned()
            }
            false => None,
        }
    }

    pub fn get_current(&self) -> Option<Arc<MoleculeWithNeighborGrid>> {
        self.molecules.get(self.current_molecule_index).cloned()
    }

    pub fn toggle_animation(&mut self) {
        self.is_animation_active = !self.is_animation_active;
    }

    pub fn set_animation_speed(&mut self, speed: u32) {
        self.animation_speed = speed;
    }

    pub fn increase_frame(&mut self) {
        if self.is_animation_active && self.molecules.len() > 1 {
            self.current_molecule_index = (self.animation_frame / 10) % self.molecules.len();
            self.update_molecule = true;
            self.animation_frame += self.animation_speed as usize;
        }
    }

    pub fn load_from_parsed(&mut self, molecules: Vec<ParsedFile>, probe_radius: f32) {
        let molecules = molecules.into_iter().map(|molecule| {
            Molecule::new(
                molecule
                    .into_iter()
                    .map(|atom| atom.into())
                    .collect::<Vec<Atom>>(),
            )
        });

        self.molecules = self.compute_neighbor_grids(molecules, probe_radius);
        self.current_molecule_index = 0;
        self.update_molecule = true;
    }

    pub fn recompute_neighbor_grids(&mut self, probe_radius: f32) {
        let molecules = self.molecules.clone();
        self.molecules = self.compute_neighbor_grids(
            molecules
                .into_iter()
                .map(|molecule| molecule.molecule.clone()),
            probe_radius,
        );

        self.update_molecule = true;
    }

    fn compute_neighbor_grids(
        &mut self,
        molecules: impl Iterator<Item = Molecule>,
        probe_radius: f32,
    ) -> Vec<Arc<MoleculeWithNeighborGrid>> {
        molecules
            .map(|molecule| {
                Arc::new(MoleculeWithNeighborGrid::from_molecule(&molecule, probe_radius))
            })
            .collect()
    }
}

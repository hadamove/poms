use std::sync::Arc;

use crate::shared::events::{AppEvent, EventDispatch};
use crate::shared::grid::MoleculeData;

use super::parse::parse_atoms_from_pdb_file;

pub struct MoleculeStore {
    pub molecules: Vec<Arc<MoleculeData>>,
    pub current_molecule_index: usize,
    dispatch: EventDispatch,
}

impl MoleculeStore {
    pub fn new(dispatch: EventDispatch) -> Self {
        Self {
            molecules: vec![],
            current_molecule_index: 0,
            dispatch,
        }
    }

    pub fn load_pdb_files_from_user(&self) {
        let dispatch = self.dispatch.clone();
        execute(async move {
            let file_dialog = rfd::AsyncFileDialog::new().add_filter("PDB", &["pdb"]);

            if let Some(files) = file_dialog.pick_files().await {
                let mut contents = Vec::new();
                for file in files {
                    contents.push(file.read().await);
                }
                dispatch.send(AppEvent::FilesLoaded(contents)).ok();
            }
        })
    }

    pub fn recompute_molecule_grids(&mut self, probe_radius: f32) {
        self.molecules = self
            .molecules
            .iter()
            .map(|molecule| {
                Arc::new(MoleculeData::from_atoms(
                    molecule.atoms_sorted.clone(),
                    probe_radius,
                ))
            })
            .collect();
    }

    fn change_current_molecule(&mut self) {
        if let Some(molecule) = self.molecules.get(self.current_molecule_index) {
            self.dispatch
                .send(AppEvent::MoleculeChanged(molecule.clone()))
                .ok();
        }
    }

    pub fn parse_molecules_and_grids(&mut self, files: Vec<Vec<u8>>, probe_radius: f32) {
        let parse_result = files
            .iter()
            .map(|file| parse_atoms_from_pdb_file(file))
            .collect::<anyhow::Result<Vec<_>>>();

        match parse_result {
            Ok(molecules) => {
                self.molecules = molecules
                    .into_iter()
                    .map(|atoms| Arc::new(MoleculeData::from_atoms(atoms, probe_radius)))
                    .collect();

                self.current_molecule_index = 0;
                self.change_current_molecule()
            }
            Err(e) => {
                self.dispatch
                    .send(AppEvent::DisplayError(e.to_string()))
                    .ok();
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn execute<F: futures::Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(move || futures::executor::block_on(f));
}

#[cfg(target_arch = "wasm32")]
fn execute<F: futures::Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

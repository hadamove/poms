use std::sync::mpsc::Sender;
use std::sync::Arc;

use crate::passes::resources::grid::GriddedMolecule;

pub enum AppEvent {
    // gui events
    SesResolutionChanged(u32),
    ProbeRadiusChanged(f32),
    RenderSpacefillChanged(bool),
    RenderSesChanged(bool),
    ComputeSesAlwaysChanged(bool),
    OpenFileDialogRequested,

    // Not gui events
    MoleculeChanged(Arc<GriddedMolecule>),
    FocusCamera(cgmath::Point3<f32>),
    DisplayError(String),
    FilesLoaded(Vec<Vec<u8>>),
}

pub type EventDispatch = Sender<AppEvent>;

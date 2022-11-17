use std::sync::mpsc::Sender;
use std::sync::Arc;

use super::grid::MoleculeData;

pub enum AppEvent {
    SesResolutionChanged(u32),
    ProbeRadiusChanged(f32),
    RenderSpacefillChanged(bool),
    RenderSesChanged(bool),
    ComputeSesAlwaysChanged(bool),
    OpenFileDialogRequested,
    MoleculeChanged(Arc<MoleculeData>),

    DisplayError(String),
    FilesLoaded(Vec<Vec<u8>>),
}

pub type EventDispatch = Sender<AppEvent>;

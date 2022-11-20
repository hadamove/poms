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
    FocusCamera(cgmath::Point3<f32>),

    DisplayError(String),
    FilesLoaded(Vec<Vec<u8>>),
}

pub type EventDispatch = Sender<AppEvent>;

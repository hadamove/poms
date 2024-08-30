mod error;
mod files;
mod menu;
mod search;
mod settings;

// Re-export the modules
pub(crate) use self::{error::*, files::*, menu::*, search::*, settings::*};

pub(crate) type UiElement = fn(&mut egui::Context, &mut super::UIState);

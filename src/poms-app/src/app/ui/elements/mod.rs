pub mod error;
pub mod files;
pub mod menu;
pub mod search;
pub mod settings;

// Re-export the modules
pub use self::{error::*, files::*, menu::*, search::*, settings::*};

pub type UiElement = fn(&egui::Context, &mut super::UIState);

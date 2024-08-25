pub mod error;
pub mod files;
pub mod menu;
pub mod settings;

// Re-export the modules
pub use self::{error::*, files::*, menu::*, settings::*};

pub type UiElement = fn(&egui::Context, &mut super::UIState);

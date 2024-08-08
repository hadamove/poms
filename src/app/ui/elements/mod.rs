pub mod error;
pub mod menu;
pub mod settings;

// Re-export the modules
pub use self::{error::*, menu::*, settings::*};

pub type UiElement = fn(&egui::Context, &mut super::UIState);

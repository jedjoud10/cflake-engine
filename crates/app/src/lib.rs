// Export
pub mod app;
mod handler;

// Re-rexport
pub use assets;
pub use audio;
pub use ecs;
pub use gui;
pub use rendering;
pub use terrain;
pub use time;
pub use world;

// Prelude
pub mod prelude {
    pub use crate::app::*;
    pub use crate::assets::*;
    pub use crate::audio::*;
    pub use crate::ecs::*;
    pub use crate::gui::*;
    pub use crate::rendering::prelude::*;
    pub use crate::terrain::*;
    pub use crate::time::*;
    pub use crate::world::*;
}
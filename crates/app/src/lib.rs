// Export
pub mod app;
mod handler;

// Re-rexport
pub use assets;
pub use audio;
pub use ecs;
pub use gui;
pub use time;
pub use rendering;
pub use resources;
pub use terrain;
pub use world;

// Prelude
pub mod prelude {
    pub use crate::app::*;
    pub use crate::assets::*;
    pub use crate::audio::*;
    pub use crate::ecs::*;
    pub use crate::gui::*;
    pub use crate::time::*;
    pub use crate::rendering::prelude::*;
    pub use crate::resources::*;
    pub use crate::terrain::*;
    pub use crate::world::*;
}

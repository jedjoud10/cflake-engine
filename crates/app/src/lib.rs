// Export
pub mod app;


// Re-rexport
pub use world;
pub use ecs;
pub use rendering;
pub use resources;
pub use audio;
pub use terrain;
pub use gui;
pub use assets;

// Prelude
pub mod prelude {
    pub use crate::app::*;
    pub use crate::world::*;
    pub use crate::ecs::*;
    pub use crate::rendering::*;
    pub use crate::resources::*;
    pub use crate::audio::*;
    pub use crate::terrain::*;
    pub use crate::gui::*;
    pub use crate::assets::*;
}
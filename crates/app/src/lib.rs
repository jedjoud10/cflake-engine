// Export
pub mod app;

// Re-rexport
pub use assets;
pub use audio;
pub use ecs;
pub use gui;
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
    pub use crate::rendering::*;
    pub use crate::resources::*;
    pub use crate::terrain::*;
    pub use crate::world::*;
}

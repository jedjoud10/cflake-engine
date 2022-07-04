// Export
pub mod app;

// Re-rexport
pub use assets;
pub use audio;
pub use ecs;
pub use gui;
pub use input;
pub use math;
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
    pub use crate::input::*;
    pub use crate::math::*;
    pub use crate::rendering::prelude::*;
    pub use crate::terrain::*;
    pub use crate::time::*;
    pub use crate::world::*;
    pub use vek;
}

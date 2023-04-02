pub mod app;
mod stats;
pub(crate) use stats::*;

pub use assets;
pub use audio;
pub use ecs;
pub use gui;
pub use input;
pub use math;
pub use networking;
pub use terrain;
pub use utils;
pub use world;
pub use coords;
pub use physics;

// Gfx related
pub use graphics;
pub use rendering;
pub mod prelude {
    pub use crate::app::*;
    pub use crate::assets::*;
    pub use crate::audio::*;
    pub use crate::ecs::*;
    pub use crate::gui::*;
    pub use crate::input::*;
    pub use crate::math::*;
    pub use crate::networking::*;
    pub use crate::terrain::*;
    pub use crate::utils::*;
    pub use crate::world::*;
    pub use crate::coords::*;
    pub use crate::physics::*;

    // Re-exports
    pub use half::f16;
    pub use log;
    pub use log::LevelFilter;
    pub use serde;
    pub use serde::{Deserialize, Serialize};
    pub use vek;
    pub use vek::{
        ops::*, Extent2, Extent3, Quaternion, Rgb, Rgba, Vec2, Vec3,
        Vec4,
    };
    pub use winit;

    // Gfx related
    pub use crate::graphics::*;
    pub use crate::rendering::*;
}

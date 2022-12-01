pub mod app;

pub use assets;
pub use audio;
pub use ecs;
pub use gui;
pub use input;
pub use math;
pub use time;
pub use utils;
pub use world;

// Gfx related
pub use graphics;
pub use rendering;
pub use vulkan;
pub mod prelude {
    pub use crate::app::*;
    pub use crate::assets::*;
    pub use crate::audio::*;
    pub use crate::ecs::*;
    pub use crate::gui::*;
    pub use crate::input::*;
    pub use crate::math::*;
    pub use crate::time::*;
    pub use crate::utils::*;
    pub use crate::world::*;

    // Re-exports
    pub use log;
    pub use vek;
    pub use vek::{Vec2, Vec3, Vec4, Quaternion, Extent2, Extent3};
    pub use winit;
    pub use serde::{Serialize, Deserialize};
    pub use serde;

    // Gfx related
    pub use crate::graphics::*;
    pub use crate::rendering::*;
    pub use crate::vulkan::*;
}

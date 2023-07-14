#![warn(missing_docs)]
#![allow(ambiguous_glob_reexports)]

//! TODO: Docs

/// TODO: Docs
pub mod app;
pub use assets;
pub use audio;
pub use coords;
pub use ecs;
//pub use gui;
pub use input;
pub use math;
pub use networking;
//pub use physics;
//pub use terrain;
pub use utils;
pub use world;
pub(crate) mod systems;

// Gfx related
pub use graphics;
//pub use rendering;

/// Prelude that re-exports most of the types and interfaces used within cFlake engine
pub mod prelude {
    pub use crate::app::*;
    pub use crate::assets::*;
    pub use crate::audio::*;
    pub use crate::coords::*;
    pub use crate::ecs::*;
    //pub use crate::gui::*;
    pub use crate::input::*;
    pub use crate::math::*;
    pub use crate::networking::*;
    //pub use crate::physics::*;
    //pub use crate::systems::camera::CameraController;
    //pub use crate::terrain::*;
    pub use crate::utils::*;
    pub use crate::world::*;

    // Re-exports
    pub use half::f16;
    pub use log;
    pub use log::LevelFilter;
    pub use serde;
    pub use serde::{Deserialize, Serialize};
    pub use vek;
    pub use vek::{ops::*, Extent2, Extent3, Quaternion, Rgb, Rgba, Vec2, Vec3, Vec4};
    pub use winit;

    // Gfx related
    pub use crate::graphics::*;
    //pub use crate::rendering::*;
}

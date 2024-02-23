pub use ::app::*;

/// Prelude that re-exports most of the types and interfaces used within cFlake engine
pub mod prelude {
    pub use crate::world::prelude::*;
    pub use crate::graphics::prelude::*;
    pub use crate::ecs::prelude::*;
    //pub use crate::rendering::prelude::*;
    pub use crate::input::prelude::*;
    pub use crate::utils::prelude::*;
    pub use crate::app::*;
    pub use crate::vek::{Vec3, Vec2, Vec4, Mat3, Mat4};
    pub use crate::vek;
    pub use crate::log;
}
#[warn(missing_docs)]

pub mod resource;
pub mod world;

pub mod prelude {
    pub use crate::resource::*;
    pub use crate::world::*;
}
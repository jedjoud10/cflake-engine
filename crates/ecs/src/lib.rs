#![warn(missing_docs)]

//! TODO: Docs

pub mod archetype;
pub mod entity;
pub mod layout;
pub mod mask;
pub mod query;
pub mod registry;
pub mod scene;
pub mod vec;
mod tests;

pub mod prelude {
    pub use crate::archetype::*;
    pub use crate::entity::*;
    pub use crate::layout::*;
    pub use crate::mask::*;
    pub use crate::query::*;
    pub use crate::registry::*;
    pub use crate::scene::*;
    pub use crate::vec::*;
}
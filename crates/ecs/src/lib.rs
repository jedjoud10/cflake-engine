#![warn(missing_docs)]

//! TODO: Docs

/// Archetypes and everything related to them
pub mod archetype;

/// Entities and their internal representation
pub mod entity;

/// Component layout, mix and matches
pub mod layout;

/// Component bitmask
pub mod mask;

/// Query items and queries themselves
pub mod query;

/// Registry to keep track of component bitmasks and TypeIds 
pub mod registry;

/// Main module that contains the Scene resource
pub mod scene;
mod tests;

/// UntypedVec utilitities
pub mod vec;

/// Re-export everything
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

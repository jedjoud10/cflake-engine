#![feature(int_roundings)]
// Modules
pub mod archetype;
pub mod component;
pub mod entity;
pub mod macros;
pub mod manager;
pub mod masks;
// Use
pub use archetype::*;
pub use component::*;
pub use entity::*;
pub use macros::*;
pub use manager::*;
pub use masks::*;
pub use utils::*;

mod tests;
pub mod utils;

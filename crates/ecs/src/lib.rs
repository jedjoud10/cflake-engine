#![feature(int_roundings)]
// Modules
mod archetype;
mod component;
mod entity;
mod macros;
mod manager;
mod masks;
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

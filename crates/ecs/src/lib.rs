#![feature(int_roundings)]
#![feature(bool_to_option)]
#![feature(test)]
// Modules
mod archetype;
mod component;
mod entity;
mod manager;
mod masks;
// Use
pub use archetype::*;
pub use component::*;
pub use entity::*;
pub use manager::*;
pub use masks::*;
pub use utils::*;

mod tests;
pub mod utils;

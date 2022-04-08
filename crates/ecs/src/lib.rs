#![feature(int_roundings)]
#![feature(bool_to_option)]
mod archetype;
mod component;
mod entity;
mod manager;
mod masks;
mod query;
mod utils;
pub use archetype::*;
pub use component::*;
pub use entity::*;
pub use manager::*;
pub use masks::*;
pub use query::*;
pub use utils::*;

mod tests;

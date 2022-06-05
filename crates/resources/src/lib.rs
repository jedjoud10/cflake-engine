#![feature(map_many_mut)]
mod resource;
mod bundle;
mod collection;
pub use resources_derive::Resource;
pub use collection::*;
pub use resource::*;
pub use bundle::*;
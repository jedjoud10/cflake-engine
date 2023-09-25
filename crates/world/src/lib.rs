#[warn(missing_docs)]

pub mod resource;
pub mod world;
pub mod plugin;
pub mod events;
pub mod system;
mod tests;

pub mod prelude {
    pub use crate::resource::*;
    pub use crate::world::*;
    pub use crate::plugin::*;
    pub use crate::system::*;
    pub use crate::events::*;
}
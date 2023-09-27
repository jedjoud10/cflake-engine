pub mod events;
pub mod plugin;
#[warn(missing_docs)]
pub mod resource;
pub mod system;
mod tests;
pub mod world;

pub mod prelude {
    pub use crate::events::*;
    pub use crate::plugin::*;
    pub use crate::resource::*;
    pub use crate::system::*;
    pub use crate::world::*;
}

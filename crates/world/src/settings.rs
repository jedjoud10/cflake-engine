mod graphics;
pub use graphics::*;
mod window;
use serde::{Deserialize, Serialize};
pub use window::*;
// Some game settings
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Settings {
}

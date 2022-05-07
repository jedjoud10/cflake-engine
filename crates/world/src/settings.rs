mod graphics;
pub use graphics::*;
mod window;
use rendering::pipeline::ShadowSettings;
use serde::{Deserialize, Serialize};
pub use window::*;
// Some game settings
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Settings {
    // Window settings
    pub window: WindowSettings,

    // Shadow settings
    pub shadows: ShadowSettings,
}

mod graphics;
pub use graphics::*;
mod window;
use rendering::pipeline::ShadowSettings;
pub use window::*;
use serde::{Serialize, Deserialize};
// Some game settings
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Settings {
    // Window settings
    pub window: WindowSettings,

    // Shadow settings
    pub shadows: ShadowSettings,
}

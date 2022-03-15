mod graphics;
pub use graphics::*;
mod window;
pub use window::*;
mod terrain;
pub use self::terrain::*;
use io::{serde, Deserialize, Serialize};
// Some game settings
#[derive(Default, Serialize, Deserialize, Clone)]
#[serde(crate = "self::serde")]
pub struct Settings {
    // Window settings
    pub window: WindowSettings,

    // Shadow settings
    pub shadows: ShadowSettings,

    // Terrain settings
    pub terrain: TerrainUserSettings,
}

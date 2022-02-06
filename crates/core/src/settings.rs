mod graphics;
use graphics::*;
use io::{serde, Deserialize, Serialize};
// Some game settings
#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct GameSettings {
    // Graphics
    #[serde(default)]
    pub vsync: bool,
    #[serde(default)]
    pub fullscreen: bool,
    #[serde(default)]
    pub shadow_resolution: ShadowResolution,
}
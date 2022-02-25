mod graphics;
use graphics::*;
use io::{serde, Deserialize, Serialize};

fn default_fps_cap() -> i32 {
    -1
}

// Some game settings
#[derive(Default, Serialize, Deserialize, Clone, Copy)]
#[serde(crate = "self::serde")]
pub struct GameSettings {
    // Graphics
    #[serde(default)]
    pub vsync: bool,
    #[serde(default)]
    pub fullscreen: bool,
    #[serde(default)]
    pub shadow_resolution: ShadowResolution,
    #[serde(default = "default_fps_cap")]
    pub fps_cap: i32,
}

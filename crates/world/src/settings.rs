mod graphics;
use graphics::*;
use io::{serde, Deserialize, Serialize};

fn default_fps_cap() -> i32 {
    -1
}

// Some game settings
#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(crate = "self::serde")]
pub struct Settings {
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

impl Default for Settings {
    fn default() -> Self {
        Self {
            vsync: Default::default(),
            fullscreen: Default::default(),
            shadow_resolution: Default::default(),
            fps_cap: default_fps_cap(),
        }
    }
}

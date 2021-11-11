use io::{serde, Deserialize, Serialize};
// The config file of the world
#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct GameConfig {
    pub vsync: bool,
    pub fullscreen: bool,
}

// Default
impl Default for GameConfig {
    fn default() -> Self {
        Self { vsync: true, fullscreen: true }
    }
}

// Terrain settings
use io::{serde, Deserialize, Serialize};

// Terrain threadness
#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "self::serde")]
#[serde(rename_all = "snake_case")]
pub enum TerrainMesherThreadingType {
    Threaded(usize),
    Single,
}

impl Default for TerrainMesherThreadingType {
    fn default() -> Self {
        Self::Threaded(2)
    }
}

#[derive(Default, Serialize, Deserialize, Clone)]
#[serde(crate = "self::serde")]
pub struct TerrainUserSettings {
    pub mesher: TerrainMesherThreadingType,
}

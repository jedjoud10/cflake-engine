// Some graphics settings
use io::{serde, Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")]
#[serde(rename_all = "snake_case")]
pub enum ShadowResolution {
    Potato,
    Low,
    Medium,
    High,
    Overkill,
    PlsHelpMyGPUIsBurning, 
}

impl Default for ShadowResolution {
    fn default() -> Self {
        Self::Medium
    }
}

impl ShadowResolution {
    // Convert to actualy shadow resolution
    pub fn convert(&self) -> u16 {
        match self {
            ShadowResolution::Potato => 256,
            ShadowResolution::Low => 512,
            ShadowResolution::Medium => 1024,
            ShadowResolution::High => 2048,
            ShadowResolution::Overkill => 4096,
            ShadowResolution::PlsHelpMyGPUIsBurning => 8192,
        }
    }
}
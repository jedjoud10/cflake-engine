// Some graphics settings
use io::{serde, Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "self::serde")]
#[serde(rename_all = "snake_case")]
pub enum ShadowResolution {
    Disabled,
    Potato,
    Low,
    Medium,
    High,
    Ultra,
}

impl Default for ShadowResolution {
    fn default() -> Self {
        Self::Medium
    }
}

impl ShadowResolution {
    // Convert to actual shadow resolution and shadow bias
    pub fn convert(&self) -> (u32, f32) {
        match self {
            ShadowResolution::Disabled => (0, 0.0),
            ShadowResolution::Potato => (512, 9.0),
            ShadowResolution::Low => (1024, 4.0),
            ShadowResolution::Medium => (2048, 3.0),
            ShadowResolution::High => (4096, 1.4),
            ShadowResolution::Ultra => (8192, 1.0),
        }
    }
}

// Shadow settings
#[derive(Default, Serialize, Deserialize, Clone)]
#[serde(crate = "self::serde")]
pub struct ShadowSettings {
    // Shadow resolution
    pub resolution: ShadowResolution,
}

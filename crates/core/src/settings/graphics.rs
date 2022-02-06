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
}

impl Default for ShadowResolution {
    fn default() -> Self {
        Self::Medium
    }
}

impl ShadowResolution {
    // Convert to actualy shadow resolution and shadow bias
    pub fn convert(&self) -> (u16, f32) {
        match self {
            ShadowResolution::Potato => (256, 0.002),
            ShadowResolution::Low => (512, 0.0008),
            ShadowResolution::Medium => (1024, 0.0006),
            ShadowResolution::High => (2048, 0.0004),
            ShadowResolution::Overkill => (4096, 0.00003),
        }
    }
}
// Some graphics settings
use io::{serde, Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde")]
#[serde(rename_all = "snake_case")]
pub enum ShadowResolution {
    Disabled,
    Potato,
    Low,
    Medium,
    High,
    Overkill,
    Stop,
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
            &ShadowResolution::Disabled => (0, 0.0),
            ShadowResolution::Potato => (512, 4.0),
            ShadowResolution::Low => (1024, 3.0),
            ShadowResolution::Medium => (2048, 2.0),
            ShadowResolution::High => (4096, 1.0),
            ShadowResolution::Overkill => (8192, 0.5),
            ShadowResolution::Stop => (16384, 0.2),
        }
    }
}

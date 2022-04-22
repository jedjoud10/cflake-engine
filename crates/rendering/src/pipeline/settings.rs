
use std::num::NonZeroU32;

use getset::{CopyGetters, Getters};
// Global pipeline settings
pub struct PipelineSettings {
    // Main shadow settings
    pub shadow: Option<ShadowSettings>,
}

// Settings specific for shadows
pub struct ShadowSettings {
    // The texture resolution of the main shadow map
    pub resolution: NonZeroU32,

    // Some bias to compesate for shadow acne
    pub bias: f32,

    // Another type of bias
    pub normal_offset: f32,

    // Used for smoothing
    pub samples: u8,
}
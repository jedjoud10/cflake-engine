
use std::num::NonZeroU32;
use serde::{Serialize, Deserialize};
use getset::{CopyGetters, Getters};
// Global pipeline settings
#[derive(CopyGetters)]
#[getset(get_copy = "pub")]
pub struct PipelineSettings {
    // Main shadow settings
    shadow: Option<ShadowSettings>,
}

impl PipelineSettings {
    // Create some new pipeline settings given the loaded settings from the user
    pub fn new(shadow: Option<ShadowSettings>) -> Self { 
        Self {
            shadow
        }
    }
}

// Settings specific for shadows
#[derive(Serialize, Deserialize, CopyGetters, Clone, Copy)]
#[getset(get_copy = "pub")]
pub struct ShadowSettings {
    // The texture resolution of the main shadow map
    resolution: u32,

    // Some bias to compesate for shadow acne
    bias: f32,

    // Another type of bias that is stronger when the surface is parallel to the light
    normal_offset: f32,

    // Used for smoothing
    samples: u8,
}

impl Default for ShadowSettings {
    fn default() -> Self {
        Self { 
            resolution: 4096,
            bias: 1.0,
            normal_offset: 0.0,
            samples: 2
        }
    }
}
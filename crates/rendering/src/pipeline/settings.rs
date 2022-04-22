// Pipeline settings
use getset::{CopyGetters, Getters};
#[derive(Getters, CopyGetters)]
pub struct PipelineSettings {
    // TODO: Generalize
    pub shadow_resolution: Option<u32>,
    pub shadow_bias: f32,
}

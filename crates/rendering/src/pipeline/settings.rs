// Pipeline settings
use getset::{CopyGetters, Getters};
#[derive(Getters, CopyGetters)]
pub struct PipelineSettings {
    pub shadow_resolution: Option<u16>,
}

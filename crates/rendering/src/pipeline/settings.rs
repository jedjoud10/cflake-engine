// Some pipeline settings that we can load from the config file
pub struct PipelineSettings {
    // Rendering
    pub shadow_resolution: u16,
    pub shadow_bias: f32,
    // Some window settings as well (fullscreen isn't present since we can update that during the frame)
    pub vsync: bool,
}

// Some info that is returned after we render a single frame
#[derive(Default)]
pub struct FrameDebugInfo {
    // Calls
    pub draw_calls: u32,
    pub shadow_draw_calls: u32,

    // Total elements drawn
    pub triangles: u64,
    pub vertices: u64,

    // Timings
    pub whole_frame: f32,
    pub render_frame: f32,
    pub eof_callbacks_execution: f32,
    pub swap_buffers: f32,
}

// Some info that is returned after we render a single frame
#[derive(Default)]
pub struct FrameDebugInfo {
    pub draw_calls: u32,
    pub shadow_draw_calls: u32,
    pub triangles: u64,
    pub vertices: u64,
}

// Render statistics that will be calculated and returned after we render the scene
#[derive(Default)]
pub struct SceneRenderStats {
    // The number of objects that were sent to the GPU to render normally
    pub drawn: usize,

    // Number of objects culled by the frustum culler
    pub culled: usize,

    // Number of shadow objects
    pub shadowed: usize,
}
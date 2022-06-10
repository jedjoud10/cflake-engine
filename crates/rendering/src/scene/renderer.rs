use ecs::Component;

// This is the main render component that we will add to entities that contain the surface components
// This render component simply tells us how exactly we should render the mesh, and if it should be culled or not
#[derive(Component)]
pub struct Renderer {
    matrix: vek::Mat4<f32>,
    visible: bool,
    //bounds: math::AABB,
}

impl Renderer {
    // Get the current mesh matrix that we will use for rendering
    pub fn matrix(&self) -> &vek::Mat4<f32> {
        &self.matrix
    }

    // Get the current visibility state of the renderer
    pub fn is_visible(&self) -> bool {
        self.visible
    }
}
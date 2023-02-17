use ecs::Component;

// This is the main render component that we will add to entities that contain the surface components
// This render component simply tells us how exactly we should render the mesh, and if it should be culled or not
#[derive(Component)]
pub struct Renderer {
    // Model matrix (translation, rotation, scale) that defines this renderer
    pub matrix: vek::Mat4<f32>,

    // Is the model currently enabled for rendering (this ignores if the model is culled or not)
    pub visible: bool,
}

impl Renderer {
    // Create a new visible entity renderer with a default matrix
    pub fn new(visible: bool, matrix: vek::Mat4<f32>) -> Self {
        Self {
            matrix, visible,
        }
    }
}


impl Default for Renderer {
    fn default() -> Self {
        Self {
            matrix: vek::Mat4::identity(),
            visible: true,
        }
    }
}

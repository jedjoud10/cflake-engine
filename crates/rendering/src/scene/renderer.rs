use ecs::Component;


// This is the main render component that we will add to entities that contain the surface components
// This render component simply tells us how exactly we should render the mesh, and if it should be culled or not
#[derive(Component)]
pub struct Renderer {
    // Model matrix (translation, rotation, scale) that defines this renderer
    matrix: vek::Mat4<f32>,

    // The current AABB bounds that this renderer will use for culling
    //bounds: AABB,

    // Is the model currently enabled for rendering (this ignores if the model is culled or not)
    enabled: bool,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            matrix: vek::Mat4::identity(),
            enabled: true,
        }
    }
}

impl Renderer {
    // Create a new entity renderer with it's raw data
    pub fn new(matrix: vek::Mat4<f32>, enabled: bool) -> Self {
        Self { matrix, enabled }
    }

    // Get the current mesh matrix that we will use for rendering
    pub fn matrix(&self) -> &vek::Mat4<f32> {
        &self.matrix
    }

    // Set the underlying mesh matrix
    pub fn set_matrix(&mut self, matrix: vek::Mat4<f32>) {
        self.matrix = matrix;
    }

    // Get the current bounds of the model
    /*
    pub fn bounds(&self) -> &AABB {
        &self.bounds
    }
    */

    // Check if we should render the model
    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

use math;

pub const DRAW_DEBUG: bool = true;
// Debug renderer functionality
// I need to reprogram this with the new multithreaded renderer
#[derive(Default)]
pub struct DebugRenderer {
}

impl DebugRenderer {    
}

// A simple debug primitives
pub struct DebugPrimitive {
    shape: math::shapes::Shape,
    tint: veclib::Vector3<f32>,
    permament: bool,
}

impl DebugPrimitive {
    // Create an empty debug primitive
    pub fn new() -> Self {
        Self {
            shape: math::shapes::Shape::new_cube(veclib::Vector3::ZERO, veclib::Vector3::ONE * 0.5),
            tint: veclib::Vector3::ONE,
            permament: true,
        }
    }
    // Set the tint of this debug primitive
    pub fn set_tint(mut self, tint: veclib::Vector3<f32>) -> Self {
        self.tint = tint;
        self
    }
    // Set the shape of this debug primitive
    pub fn set_shape(mut self, shape: math::shapes::Shape) -> Self {
        self.shape = shape;
        self
    }
    // Set the lifetime of this debug primitive
    pub fn set_lifetime(mut self, permament: bool) -> Self {
        self.permament = permament;
        self
    }
}

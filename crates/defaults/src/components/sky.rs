use ecs::*;

// A component that will be linked to the skysphere
#[derive(Default)]
pub struct Sky {
    pub sky_gradient_texture_id: usize,
}

// Main traits implemented
ecs::impl_component!(Sky);

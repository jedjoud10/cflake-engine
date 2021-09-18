use ecs::*;

// A component that will be linked to the skysphere
#[derive(Default)]
pub struct Sky {
    pub sky_gradient_texture_id: u16,
}

// Main traits implemented
ecs::impl_component!(Sky);

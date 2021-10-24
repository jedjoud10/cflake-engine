use ecs::*;

// A component that will be linked to the skysphere
#[derive(Default)]
pub struct Sky {
}

// Main traits implemented
ecs::impl_component!(Sky);

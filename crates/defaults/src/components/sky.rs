use ecs::*;

// A component that will be linked to the skysphere
#[derive(Default)]
pub struct Sky {
    // Light uniform index
    pub light_dir_index: usize,
}

// Main traits implemented
ecs::impl_component!(Sky);

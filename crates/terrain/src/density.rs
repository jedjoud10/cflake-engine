// Create a compute shader directly from Rust code
pub struct Density {
    // The ID of the current density node
    // The ID of the last density that is on the density graph
    // The type of density node
    // Strengh and scale
    pub strength: f32,
    pub scale: f32,
}

// Density node type
pub enum DensityType {
    Position(veclib::Vec3Axis),
    Sin(f32),
    Cos(f32),
    Perlin(),
    Cellular(bool),
}

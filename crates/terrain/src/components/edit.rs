// Terrain edit "modes"
pub enum EditMode {
    // Adds the terrain edit into the terrain
    Addition,

    // Adds the terrain edit into the terrain smoothly using a factor
    AdditionSmoothed(f32),

    // Subtracts the terrain edit from the terrain
    Subtraction,

    // Subtracts the terrain edit from the terrain smoothly using a factor
    SubtractionSmoothed(f32),
}

// The shape of the terrain edit
// Shape origin is added to the edit entity transformation
pub enum EditShape {
    Cuboid(math::shapes::Cuboid<f32>),
    Sphere(math::shapes::Sphere<f32>),
}

// A terrain edit can be created by spawning in an entity that contains the components with optional location/rotation/scale components
pub struct Edit {
    // How the edit will affect the terrain
    pub mode: EditMode,

    // The shape of the terrain edit
    pub shape: EditShape,

    // Custom color if we wish to override the color of the terrain
    pub color: Option<vek::Rgb<f32>>,
}

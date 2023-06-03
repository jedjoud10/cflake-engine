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
    pub color: Option<vek::Rgb<u8>>,
}

// Rust representation of the GLSL packed edit struct
#[repr(C)]
pub struct PackedEdit {
    mode: u32,
    smoothing: f32,
    shape: u32,
    color: vek::Vec4<u8>,
    center: vek::Vec4<f32>,
    extra: vek::Vec4<f32>,
}

// Convert a normal edit to a packet edit
pub(crate) fn pack(edit: Edit) -> PackedEdit {
    let (mode, smoothing) = match edit.mode {
        EditMode::Addition => (1, 0.0),
        EditMode::AdditionSmoothed(x) => (2, x),
        EditMode::Subtraction => (3, 0.0),
        EditMode::SubtractionSmoothed(x) => (4, x),
    };

    let (shape, center, extra) = match edit.shape {
        EditShape::Cuboid(cuboid) => (1, cuboid.center.with_w(0.0), vek::Vec3::<f32>::from(cuboid.half_extent).with_w(0.0)),
        EditShape::Sphere(sphere) => (2, sphere.center.with_w(0.0), vek::Vec4::new(sphere.radius, 0.0, 0.0, 0.0)),
    };

    PackedEdit {
        mode,
        smoothing,
        shape,
        color: vek::Vec4::one(),
        center,
        extra
    }
}
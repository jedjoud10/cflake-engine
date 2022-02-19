use math::shapes::BasicShapeType;

// A single terrain edit
pub struct Edit {
    // Contains the shape of the edit and some other edit parameters
    pub shape: BasicShapeType,

    // Params
    pub strength: f32,
    pub material: Option<u8>,
}

impl Edit {
    // Create a new edit
    pub fn new(shape: BasicShapeType, strength: f32) -> Self {
        Self { shape, strength, material: None }
    }
    // This edit contains a specific material override
    pub fn with_material(mut self, material: u8) -> Self {
        self.material = Some(material);
        self
    }
}

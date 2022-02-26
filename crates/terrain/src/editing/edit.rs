use math::{csg::CSGOperation, shapes::ShapeType};

// A single terrain edit
#[derive(Clone)]
pub struct Edit {
    // Contains the shape of the edit and some other edit parameters
    pub shape: ShapeType,

    // Params
    pub material: Option<u8>,
    pub color: veclib::Vector3<u8>,
    pub operation: CSGOperation,
}

impl Edit {
    // Create a new edit
    pub fn new(shape: ShapeType, operation: CSGOperation) -> Self {
        Self {
            shape,
            material: None,
            color: veclib::Vector3::ONE * 255,
            operation,
        }
    }
    // Parameters
    pub fn with_material(mut self, material: u8) -> Self {
        self.material = Some(material);
        self
    }
    pub fn with_color(mut self, color: veclib::Vector3<u8>) -> Self {
        self.color = color;
        self
    }
}

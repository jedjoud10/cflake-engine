use rendering::{
    basics::{shader::Shader, texture::Texture},
    object::ObjectID,
};

use crate::ElementID;

// A simple element, could be a button or a panel or anything, it just has some variables
#[derive(Debug, Clone)]
pub struct Element {
    // Indexing
    pub id: Option<ElementID>,
    pub parent: Option<ElementID>,
    pub children: Vec<ElementID>,

    // Center and scale (in pixels)
    pub center: veclib::Vector2<u16>,
    pub size: veclib::Vector2<u16>,
    // Rendering
    pub color: veclib::Vector4<f32>,
    pub visible: bool,
    pub depth: i32,
    pub texture: ObjectID<Texture>,
    // [X, Y] for min, [Z, W] for max
    pub texture_uvs: veclib::Vector4<f32>,
    pub shader: ObjectID<Shader>,

    // Others
    pub _type: ElementType,
}

impl Default for Element {
    fn default() -> Self {
        Self {
            id: None,
            parent: None,
            children: Vec::new(),

            center: veclib::Vector2::ZERO,
            size: veclib::Vector2::ONE,

            color: veclib::Vector4::ONE,
            visible: true,
            depth: 0,
            texture: Default::default(),
            texture_uvs: veclib::vec4(0.0, 0.0, 1.0, 1.0),
            shader: Default::default(),

            _type: ElementType::Panel,
        }
    }
}

// Code for generation of the elements
impl Element {
    // Create a new element with default parameters
    pub fn new() -> Self {
        Self::default()
    }
    // Set the center of the element
    pub fn with_center(mut self, center: veclib::Vector2<u16>) -> Self {
        self.center = center;
        self
    }
    // Set the size of the element
    pub fn with_size(mut self, size: veclib::Vector2<u16>) -> Self {
        self.size = size;
        self
    }
    // Set the type of the element
    pub fn with_type(mut self, _type: ElementType) -> Self {
        self._type = _type;
        self
    }
    // Set the color of the element
    pub fn with_color(mut self, color: veclib::Vector4<f32>) -> Self {
        self.color = color;
        self
    }
    // Set the visibility of the element
    pub fn with_visibility(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

// The type of element
#[derive(Debug, Clone)]
pub enum ElementType {
    Root,
    Panel,
    Text(String),
}

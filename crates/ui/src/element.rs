use rendering::{object::ObjectID, basics::{texture::Texture, shader::Shader}};

use crate::{ElementID};

// A simple element, could be a button or a panel or anything, it just has some variables
#[derive(Debug, Clone)]
pub struct Element {
    // Indexing
    pub id: Option<ElementID>,
    pub parent: Option<ElementID>,
    pub children: Vec<ElementID>,

    // Position and scale
    pub position: veclib::Vector2<f32>,
    pub size: veclib::Vector2<f32>,
    // Rendering
    pub color: veclib::Vector4<f32>,
    pub visible: bool,
    pub depth: i32,
    pub texture: ObjectID<Texture>,
    pub shader: ObjectID<Shader>,

    // Others
    pub _type: ElementType,
    pub coords: CoordinateType,
}

impl Default for Element {
    fn default() -> Self {
        Self {
            id: None,
            parent: None,
            children: Vec::new(),

            position: veclib::Vector2::ZERO,
            size: veclib::Vector2::ONE,

            color: veclib::Vector4::ONE,
            visible: true,
            depth: 0,
            texture: Default::default(),
            shader: Default::default(),

            _type: ElementType::Panel,
            coords: CoordinateType::Factor,
        }
    }
}

// Code for generation of the elements
impl Element {
    // Create a new element with default parameters
    pub fn new() -> Self {
        Self::default()
    }
    // Set the coordinate system for this element
    pub fn set_coordinate_system(mut self, coords: CoordinateType) -> Self {
        self.coords = coords;
        self
    }
    // Set the position of the element
    pub fn set_position(mut self, position: veclib::Vector2<f32>) -> Self {
        self.position = position;
        self
    }
    // Set the size of the element
    pub fn set_size(mut self, size: veclib::Vector2<f32>) -> Self {
        self.size = size;
        self
    }
    // Set the type of the element
    pub fn set_type(mut self, _type: ElementType) -> Self {
        self._type = _type;
        self
    }
    // Set the color of the element
    pub fn set_color(mut self, color: veclib::Vector4<f32>) -> Self {
        self.color = color;
        self
    }
    // Set the visibility of the element
    pub fn set_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
    // ----Update functions----
    pub fn update_text(&mut self, text: &str) {
        self._type = ElementType::Text(text.to_string());
    }
}

// Coordinate type
#[derive(Debug, Clone)]
pub enum CoordinateType {
    Pixel,
    Factor,
}

// The type of element
#[derive(Debug, Clone)]
pub enum ElementType {
    Root,
    Panel,
    Text(String),
}

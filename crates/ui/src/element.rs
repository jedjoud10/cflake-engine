use crate::{ElementID};

// A simple element, could be a button or a panel or anything, it just has some variables
#[derive(Debug, Clone)]
pub struct Element {
    // Indexing
    pub id: ElementID,
    pub parent: ElementID,

    // Position and scale
    pub position: veclib::Vector2<f32>,
    pub size: veclib::Vector2<f32>,
    
    // Our color in RGBA form
    pub color: veclib::Vector4<f32>,
    // If the element is even visible, this propagates down to it's children
    pub visible: bool,
    // The depth of this node, further depth nodes get rendered front to back
    pub depth: i32,
    // Our children
    pub children: Vec<ElementID>,
    pub _type: ElementType,
    // Coordinate system type
    pub coords: CoordinateType,
}

impl Default for Element {
    fn default() -> Self {
        Self {
            // Parent stuff
            id: ElementID(None),
            parent: ElementID(None),
            // Data
            position: veclib::Vector2::ZERO,
            size: veclib::Vector2::ONE,
            color: veclib::Vector4::ONE,
            visible: true,
            coords: CoordinateType::Factor,
            _type: ElementType::Panel,
            // Internal data
            depth: 0,
            children: Vec::new(),
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

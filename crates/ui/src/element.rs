use crate::Root;

// A simple element, could be a button or a panel or anything, it just has some variables
#[derive(Debug)]
pub struct Element {
    // The ID of this element in the root node
    pub id: usize,
    // Our parent node's ID
    pub parent: usize,
    // Our position relative to our coordinate system
    pub position: veclib::Vector2<f32>,
    // Our size relative to our coordinate system
    pub size: veclib::Vector2<f32>,
    // Our color in RGBA form
    pub color: veclib::Vector4<f32>,
    // The depth of this node, further depth nodes get rendered front to back
    pub depth: i32,
    pub children: Vec<usize>,
    pub element_type: ElementType,
    // Coordinate system type
    pub coordinate_type: CoordinateType,
}

impl Default for Element {
    fn default() -> Self {
        Self {
            // Parent stuff
            id: 0,
            parent: 0,
            // Data
            position: veclib::Vector2::ZERO,
            size: veclib::Vector2::ONE,
            color: veclib::Vector4::ONE,
            coordinate_type: CoordinateType::Factor,
            element_type: ElementType::Empty,
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
        return Self::default();
    }
    // Set the coordinate system for this element
    pub fn set_coordinate_system(mut self, coordinate_type: CoordinateType) -> Self {
        self.coordinate_type = coordinate_type;
        return self;
    }
    // Set the position of the element
    pub fn set_position(mut self, position: veclib::Vector2<f32>) -> Self {
        self.position = position;
        return self;
    }
    // Set the size of the element
    pub fn set_size(mut self, size: veclib::Vector2<f32>) -> Self {
        self.size = size;
        return self;
    }
    // Set the text of the element
    pub fn set_text(mut self, text: &str) -> Self {
        // Set the type of element
        self.element_type = ElementType::Text(text.to_string());
        return self;
    }
}

// Coordinate type
#[derive(Debug)]
pub enum CoordinateType {
    Pixel,
    Factor,
}

// The state of a button element
#[derive(Debug)]
pub enum ButtonState {
    Pressed,
    Released,
    Held,
}

// The type of element
#[derive(Debug)]
pub enum ElementType {
    Empty,
    Panel(),
    Button(ButtonState),
    Text(String),
    Image(String),
}

impl Element {
    // Attach children to this element
    pub fn attach(root: &mut Root, id: usize, children: Vec<usize>) {
        // Update the parent id of the children
        for child_element_id in children.iter() {
            // Get the child element and update it
            let child_element = root.smart_element_list.get_element_mut(&(*child_element_id as u16)).unwrap();
            child_element.parent = id;
            child_element.depth += 1;
            root.max_depth = root.max_depth.max(child_element.depth);
        }
        let element = root.smart_element_list.get_element_mut(&(id as u16)).unwrap();
        element.children.extend(children);
    }
}

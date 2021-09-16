use crate::Root;

// A simple element, could be a button or a panel or anything, it just has some variables
#[derive(Debug)]
pub struct Element {
    pub id: usize,
    pub parent: usize,
    pub position: veclib::Vector2<f32>,
    pub size: veclib::Vector2<f32>,
    pub color: veclib::Vector4<f32>,
    pub depth: i32,
    pub children: Vec<usize>,
    pub element_type: ElementType,
    pub coordinate_type: CoordinateType,
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
    // Create a new element
    pub fn new(
        root: &mut Root,
        position: &veclib::Vector2<f32>,
        size: &veclib::Vector2<f32>,
        color: &veclib::Vector4<f32>,
        element_type: ElementType,
        coordinate_type: CoordinateType,
    ) -> usize {
        // Get the element id from the root node
        let output: Self = Element {
            size: size.clone(),
            position: position.clone(),
            parent: 0,
            id: root.smart_element_list.get_next_valid_id() as usize,
            depth: 0,
            children: Vec::new(),
            element_type: element_type,
            color: color.clone(),
            coordinate_type,
        };
        // Attach this element to the root element
        let output_id = root.add_element(output) as usize;
        if output_id != 0 {
            Element::attach(root, 0, vec![output_id]);
        }

        // Add the element
        return output_id;
    }
}

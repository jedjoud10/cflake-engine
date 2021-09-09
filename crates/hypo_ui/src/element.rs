use crate::Root;

// A simple element, could be a button or a panel or anything, it just has some variables
pub struct Element {
    pub size: veclib::Vector2<f32>,
    pub position: veclib::Vector2<f32>,
    pub parent: usize,
    id: usize,
    pub children: Vec<usize>,    
}

impl Element {
    // Attach children to this element
    pub fn attach(&mut self, children: Vec<Element>) {
        self.children = children.iter().map(|x| x.id).collect();
    }
    // Create a new element
    pub fn new(root: &mut Root, position: &veclib::Vector2<f32>, size: &veclib::Vector2<f32>) {
        // Get the element id from the root node
        let output: Self = Element { size: size.clone(), position: position.clone(), parent: 0, id: root, children: () }
    }
}
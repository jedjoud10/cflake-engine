use crate::Element;

// The root UI element on the screen, contains all the elements in a binary tree fashion
pub struct Root {
    pub elements: Vec<Option<Element>>,
}

impl Root {
    // Get the next free spot
    // Add an element to the tree
    pub fn add_element(&mut self, element: Element) {
        self.elements.push(Some(element));
    }
    // Remove an element from the three, and recursively remove it's children
    pub fn remove_element(&mut self, element: Element) {
        // Get all the children from this element, recursively 
        let mut output_elem_indices: Vec<usize> = Vec::new();
        let mut elems_to_evaluate: Vec<usize> = Vec::new();
        elems_to_evaluate.extend(element.children);
        while elems_to_evaluate.len() > 0 {
            // We need to get the children of this element
            let elem = self.elements.get(elems_to_evaluate[0]).unwrap();
            let children = elem.as_ref().unwrap().children.clone();
            elems_to_evaluate.extend(children);
            elems_to_evaluate.remove(0);
        }
    }
}
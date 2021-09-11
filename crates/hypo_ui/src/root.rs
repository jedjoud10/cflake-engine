use crate::Element;
use hypo_others::SmartList;

// The root UI element on the screen, contains all the elements in a binary tree fashion
#[derive(Default)]
pub struct Root {
    pub smart_element_list: SmartList<Element>,
    pub max_depth: i32,
}

impl Root {
    // Get the next free spot
    // Add an element to the tree
    pub fn add_element(&mut self, element: Element) -> usize {
        return self.smart_element_list.add_element(element) as usize;
    }
    // Remove an element from the three, and recursively remove it's children
    pub fn remove_element(&mut self, element: Element) {
        // Get all the children from this element, recursively
        let mut output_elem_indices: Vec<usize> = Vec::new();
        let mut elems_to_evaluate: Vec<usize> = Vec::new();
        elems_to_evaluate.extend(element.children);
        while elems_to_evaluate.len() > 0 {
            // We need to get the children of this element
            let elem = self.smart_element_list.get_element(&(elems_to_evaluate[0] as u16)).unwrap();
            let children = elem.children.clone();
            elems_to_evaluate.extend(children);
            elems_to_evaluate.remove(0);
        }
    }
}

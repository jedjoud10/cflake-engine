use std::collections::HashMap;

use crate::ButtonState;
use crate::CoordinateType;
use crate::Element;
use crate::ElementType;
use others::SmartList;
use resources::LoadableResource;
use resources::Resource;

// The root UI element on the screen, contains all the elements in a binary tree fashion
#[derive(Debug, Clone)]
pub struct Root {
    pub smart_element_list: SmartList<Element>,
    pub visible: bool,
    pub max_depth: i32,
    pub root_depth: i32,
}

impl Default for Root {
    fn default() -> Self {
        Self {
            smart_element_list: SmartList::<Element>::default(),
            visible: true,
            max_depth: 0,
            root_depth: 1,
        }
    }
}

impl Root {
    // New
    pub fn new(root_depth: i32) -> Self {
        let mut root = Self::default();
        // Add the root element to this
        let root_elem = Element::default();
        // Add the element to the root node
        root.add_element(root_elem);
        root.root_depth = root_depth;
        return root;
    }
    // Add an element to the tree
    pub fn add_element(&mut self, mut element: Element) -> usize {
        // Get the ID of the element
        let element_id = self.smart_element_list.get_next_valid_id();
        element.id = element_id;
        element.depth += 1;
        // Add the element
        let element_id = self.smart_element_list.add_element(element);
        // Attach this element to the root element
        Element::attach(self, 0, vec![element_id]);
        return element_id;
    }
    // Remove an element from the three, and recursively remove it's children
    pub fn remove_element(&mut self, element: Element) {
        // Get all the children from this element, recursively
        let mut output_elem_indices: Vec<usize> = Vec::new();
        let mut elems_to_evaluate: Vec<usize> = Vec::new();
        elems_to_evaluate.extend(element.children);
        while elems_to_evaluate.len() > 0 {
            // We need to get the children of this element
            let elem = self.smart_element_list.get_element(elems_to_evaluate[0]).unwrap().unwrap();
            let children = elem.children.clone();
            elems_to_evaluate.extend(children);
            elems_to_evaluate.remove(0);
        }
    }

    // ---- Actual root UI stuff ---- \\
    // Get an element from the root using it's id
    pub fn get_element(&self, id: usize) -> &Element {
        self.smart_element_list.get_element(id).unwrap().unwrap()
    }
    // Get an element from the root using it's id
    pub fn get_element_mut(&mut self, id: usize) -> &mut Element {
        self.smart_element_list.get_element_mut(id).unwrap().unwrap()
    }
    // Get the state of a specific button element
    pub fn get_button_state(&self, element_id: usize) -> &ButtonState {
        // Get the element
        let elem = self.smart_element_list.get_element(element_id).unwrap().unwrap();
        let state = match elem.element_type {
            ElementType::Button(ref state) => state,
            _ => &ButtonState::Released,
        };
        return state;
    }
    // Set the text of a text element
    pub fn set_text_state(&mut self, element_id: usize, text: &str) {
        // Get the element mutably
        let elem = self.smart_element_list.get_element_mut(element_id).unwrap().unwrap();
        match elem.element_type {
            ElementType::Text(ref mut last_text, font_size) => {
                // Set the text
                *last_text = text.to_string();
            }
            _ => {}
        }
    }
}

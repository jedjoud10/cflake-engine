use std::collections::HashMap;

use crate::ButtonState;
use crate::CoordinateType;
use crate::Element;
use crate::ElementType;
use others::SmartList;
use resources::LoadableResource;
use resources::Resource;

// The root UI element on the screen, contains all the elements in a binary tree fashion
#[derive(Default, Debug)]
pub struct Root {
    pub smart_element_list: SmartList<Element>,
    pub max_depth: i32,
}

impl Root {
    // New
    pub fn new() -> Self {
        Self::default()
    }
    // Add an element to the tree
    pub fn add_element(&mut self, mut element: Element) -> usize {
        // Get the ID of the element
        let element_id = self.smart_element_list.get_next_valid_id() as usize;
        element.id = element_id;
        element.depth += 1;
        // Add the element
        let element_id = self.smart_element_list.add_element(element) as usize;
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
            let elem = self.smart_element_list.get_element(elems_to_evaluate[0] as u16).unwrap();
            let children = elem.children.clone();
            elems_to_evaluate.extend(children);
            elems_to_evaluate.remove(0);
        }
    }

    // ---- Actual root UI stuff ---- \\
    // Get an element from the root using it's id
    pub fn get_element(&self, id: u16) -> &Element { 
        self.smart_element_list.get_element(id).unwrap()
    }
    // Get an element from the root using it's id
    pub fn get_element_mut(&mut self, id: u16) -> &mut Element { 
        self.smart_element_list.get_element_mut(id).unwrap()
    }
    // Get the state of a specific button element
    pub fn get_button_state(&self, element_id: u16) -> &ButtonState {
        // Get the element
        let elem = self.smart_element_list.get_element(element_id).unwrap();
        let state = match elem.element_type {
            ElementType::Button(ref state) => state,
            _ => &ButtonState::Released,
        };
        return state;
    }
    // Set the text of a text element
    pub fn set_text_state(&mut self, element_id: u16, text: &str) {
        // Get the element mutably
        let elem = self.smart_element_list.get_element_mut(element_id).unwrap();
        match elem.element_type {
            ElementType::Text(ref mut last_text, font_size) => {
                // Set the text
                *last_text = text.to_string();
            }
            _ => {}
        }
    }
}

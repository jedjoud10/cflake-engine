use std::collections::HashMap;

use crate::ButtonState;
use crate::Element;
use crate::ElementType;
use hypo_others::SmartList;
use hypo_resources::LoadableResource;
use hypo_resources::Resource;

// The root UI element on the screen, contains all the elements in a binary tree fashion
#[derive(Default, Debug)]
pub struct Root {
    pub smart_element_list: SmartList<Element>,
    pub max_depth: i32,
}

// Loadable resource
impl LoadableResource for Root {
    // Turn the LoadedUIRoot into this Root struct
    fn from_resource(self, resource: &hypo_resources::Resource) -> Option<Self> where Self: Sized {
        match resource {
            Resource::UIRoot(root, _) => {
                let mut output_root: Root = Root::default();
                
                // Create the root element
                Element::new(&mut output_root, &veclib::Vector2::ZERO, &veclib::Vector2::ONE, &veclib::Vector4::ZERO, ElementType::Empty);

                // The list of children-parent links
                let mut parent_children: HashMap<usize, Vec<usize>> = HashMap::new();

                for loaded_element in root.elements.iter() {
                    let element_type = match &loaded_element.loaded_elem_type {
                        hypo_resources::LoadedUIElementType::Panel() => ElementType::Panel(),
                        hypo_resources::LoadedUIElementType::Button() => ElementType::Button(ButtonState::Released),
                        hypo_resources::LoadedUIElementType::Text(t) => ElementType::Text(t.clone()),
                        hypo_resources::LoadedUIElementType::Image(lp) => ElementType::Image(lp.clone()),
                    };
                    println!("{:?}", loaded_element);
                    let element = Element::new(&mut output_root, &loaded_element.pos, &loaded_element.size, &loaded_element.color, element_type);
                    // Attach this specific element to it's valid parent
                    if loaded_element.pid != 0 {
                        // Add this child into the children of the same parent
                        let old_children = parent_children.entry(loaded_element.pid as usize).or_insert(Vec::new());
                        old_children.push(loaded_element.id as usize);
                    }
                }
                println!("{:?}", parent_children);
                // Link all the children to the parents
                for (parent, children) in parent_children {
                    Element::attach(&mut output_root, parent, children)
                }
                let t=  output_root.smart_element_list.elements.iter().map(|x| x.as_ref().unwrap().parent).collect::<Vec<usize>>();
                for i in output_root.smart_element_list.elements {
                    match i {
                        Some(x) => {
                            println!("{:?}", x);
                        }
                        _ => {}
                    }
                }
                panic!("");
                Some(output_root)
            }
            _ => { /* We are doomed */ None }
        }        
    }
}

impl Root {
    // New
    pub fn new() -> Self {
        Self::default()
    }
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
    // Load a specific UI file from the resources 
    pub fn load_root_file(local_path: &str) -> Root {
        let mut output: Root = Root::default();
        
        return output;
    }
}

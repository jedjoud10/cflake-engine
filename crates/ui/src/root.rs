use std::collections::HashMap;
use std::collections::HashSet;

use crate::Element;
use crate::ElementID;
use crate::ElementType;
use crate::InstancedBatchIdentifier;
use ordered_vec::simple::OrderedVec;
use rendering::basics::shader::Shader;
use rendering::object::ObjectID;

// The root UI element on the screen, contains all the elements in a binary tree fashion
pub struct Root {
    // All of the UI elements that will be drawn
    pub elements: OrderedVec<Element>,
    // Is the root even visible?
    pub visible: bool,
    // The elements that we have added, replaced, and removed from the root this frame
    pub added: HashSet<ElementID>,
    pub mutated: HashSet<ElementID>,
    pub removed: HashMap<ElementID, InstancedBatchIdentifier>,
}

impl Default for Root {
    fn default() -> Self {
        // Create the root
        let mut root = Self {
            elements: OrderedVec::<Element>::default(),
            visible: true,
            added: HashSet::with_capacity(8),
            mutated: HashSet::with_capacity(8),
            removed: HashMap::with_capacity(8),
        };
        // And add the root element to it
        root.add_element(Element {
            id: Some(root.root()),
            parent: None,
            ..Default::default()
        });
        root
    }
}

impl Root {
    // Get the ElementID of the root element
    pub fn root(&self) -> ElementID {
        ElementID(0)
    }
    // Add an element to the tree
    pub fn add_element(&mut self, mut element: Element) -> ElementID {
        // Get the ID of the element
        let id = ElementID(self.elements.get_next_id());
        element.id = Some(id);
        element.depth += 1;
        // Add the element
        self.elements.push_shove(element);
        // And also add the element to our root
        self.attach(self.root(), &[id]);
        // Update diffs
        self.added.insert(id);
        id
    }
    // Remove an element from the three, and recursively remove it's children
    pub fn remove_element(&mut self, id: ElementID) -> Option<()> {
        // We cannot remove the root element
        if id == self.root() {
            return None;
        }

        // Recursively get the children if we need to
        let element = self.get_element(id)?;
        let shader = element.shader;
        let texture = element.texture;
        let batch_id = InstancedBatchIdentifier { shader, texture };
        let recurse = !element.children.is_empty();
        drop(element);
        if recurse {
            let mut pending: Vec<ElementID> = vec![id];
            while let Some(parent_id) = pending.pop() {
                // Get all of our children and add them, whilst removing self
                pending.extend(self.get_element(parent_id)?.children.clone());
                self.remove_element(id)?;
            }
        }
        // Update diffs
        self.removed.insert(id, batch_id);
        self.added.remove(&id);
        self.mutated.remove(&id);
        Some(())
    }
    // Attach some child elements to an element
    pub fn attach(&mut self, id: ElementID, children: &[ElementID]) -> Option<()> {
        // Get the element first and update it's local children
        let elem = self.get_element_mut(id)?;
        let depth = elem.depth;
        elem.children.extend_from_slice(children);
        // Update the parent ID and depth of every child
        for child_id in children {
            let child = self.get_element_mut(*child_id)?;
            child.parent = Some(id);
            child.depth = depth + 1;
        }
        Some(())
    }
    // Calculate the max depth
    pub fn calculate_max_depth(&self) -> i32 {
        let element = self.elements.iter().max_by_key(|(x, element)| element.depth);
        element.map(|(_, element)| Some(element.depth)).flatten().unwrap_or_default()
    }
    // Get an element from the root using it's id
    pub fn get_element(&self, id: ElementID) -> Option<&Element> {
        self.elements.get(id.0)
    }
    // Get a mutable element from the root using it's id
    pub fn get_element_mut(&mut self, id: ElementID) -> Option<&mut Element> {
        // Update diffs
        self.mutated.remove(&id);
        self.elements.get_mut(id.0)
    }
}

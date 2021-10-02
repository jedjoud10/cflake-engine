use std::ops::{Index, IndexMut};

use fonts::FontManager;

use crate::{element::ButtonState, element::ElementType, Root};

// The UI manager, it can contain multiple UI roots, and switch between them
#[derive(Default)]
pub struct UIManager {
    pub roots: Vec<Root>,
    pub font_manager: FontManager,
}

// Actually UI functions
impl UIManager {
    // Check if we have a default root set
    pub fn default_root_valid(&self) -> bool {
        self.roots.len() > 0
    }
    // Get the default root
    pub fn get_default_root(&self) -> &Root {
        self.roots.get(0).unwrap()
    }
    pub fn get_default_root_mut(&mut self) -> &mut Root {
        self.roots.get_mut(0).unwrap()
    }
    // Set the default UI root that will be drawn on the screen by default
    pub fn set_default_root(&mut self, root: Root) {
        if self.roots.len() == 0 {
            // If this root does not exist, add it
            self.roots.push(root);
        } else {
            // If it does exist, just update it
            self.roots[0] = root;
        }
    }
}

// Borrow indexer
impl Index<usize> for UIManager {
    type Output = Root;

    fn index(&self, index: usize) -> &Self::Output {
        self.roots.get(index).unwrap()
    }
}
// Borrow mut indexer
impl IndexMut<usize> for UIManager {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.roots.get_mut(index).unwrap()
    }
}

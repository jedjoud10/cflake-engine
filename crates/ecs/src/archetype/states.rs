use std::cell::RefCell;

use crate::Mask;

// Component states (their mutation state)
#[derive(Default)]
pub struct ComponentStateSet {
    rows: RefCell<Vec<Mask>>,
}

impl ComponentStateSet {
    // Reset all the rows to a specific state
    pub fn reset_to(&self, state: bool) {
        let def = if state { Mask::all() } else { Mask::zero() };
        self.rows.borrow_mut().iter_mut().for_each(|x| *x = def);
    }
    // Reserve enough capacity to hold "additional" more elements
    pub fn reserve(&self, additional: usize) {
        self.rows.borrow_mut().reserve(additional);
    }
    // Adds a new component row
    pub fn push(&self) {
        self.rows.borrow_mut().push(Mask::zero());
    }
    // Set all the component states for a single bundle at the same time
    pub fn set(&self, bundle: usize, mask: Mask) -> Option<()> {
        // Get the row
        let mut borrowed = self.rows.borrow_mut();
        let row = borrowed.get_mut(bundle)?;

        // Overwrite the bit
        // This works for both layout masks and component masks
        *row = *row | mask;
        Some(())
    }
    // Get all the component states for a specific bundle
    pub fn get(&self, bundle: usize) -> Option<Mask> {
        let borrowed = self.rows.borrow();
        borrowed.get(bundle).cloned()
    }
}

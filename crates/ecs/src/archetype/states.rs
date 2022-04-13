use std::cell::RefCell;

use crate::Mask;

// Component states (their mutation state)
#[derive(Default)]
pub struct ComponentStateSet {
    rows: RefCell<Vec<u64>>,
}

impl ComponentStateSet {
    // Reset all the rows to a specific state
    pub fn reset_to(&self, state: bool) {
        let def = if state { u64::MAX } else { 0 };
        self.rows.borrow_mut().iter_mut().for_each(|x| *x = def);
    }
    // Reserve enough capacity to hold "additional" more elements
    pub fn reserve(&self, additional: usize) {
        self.rows.borrow_mut().reserve(additional);
    }
    // Adds a new component row
    pub fn push(&self) {
        self.rows.borrow_mut().push(0);
    }
    // Set a component state to true by bitshifting
    pub fn set(&self, bundle: usize, mask: Mask) -> Option<()> {
        // Get the row
        let mut borrowed = self.rows.borrow_mut();
        let row = borrowed.get_mut(bundle)?;

        // Overwrite the bit
        // This works for both layout masks and component masks
        *row |= mask.0;
        Some(())
    }
    // Read a component state by bitshifting
    pub fn get(&self, bundle: usize, mask: Mask) -> Option<bool> {
        let borrowed = self.rows.borrow();
        let offset = mask.offset();
        let row = borrowed.get(bundle)?;
        Some(((*row >> offset) & 1) == 1)
    }
}

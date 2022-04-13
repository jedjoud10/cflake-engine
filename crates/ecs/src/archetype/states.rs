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
    // Set a component state by bitshifting
    // This will return the old state value at that index
    pub fn set(&self, state: bool, bundle: usize, mask: Mask) -> Option<()> {
        // Get the row
        let mut borrowed = self.rows.borrow_mut();
        let offset = mask.offset();
        let old_row = borrowed.get_mut(bundle)?;

        // Overwrite the bit
        *old_row &= !(1 << offset);
        *old_row |= (state as u64) << offset;
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

use std::cell::RefCell;

// Component states (their mutation state)
#[derive(Default)]
pub struct ComponentStates {
    rows: RefCell<Vec<u64>>,
}

impl ComponentStates {
    // Reserve enough space to hold "additional" more elements
    pub fn reserve(&self, additional: usize) {
        self.rows.borrow_mut().reserve(additional);
    }
    // Set a component state by bitshifting
    // This will return the old state value at that index
    pub fn set(&self, state: bool, bundle: usize) -> Option<bool> {
        // Get the row
        let mut borrowed = self.rows.borrow_mut();
        let old_row = borrowed.get_mut(bundle)?;
        let old_state = ((*old_row >> bundle) & 1) == 1; 
        
        // Overwrite the bit
        *old_row &= !(1 << bundle); 
        *old_row |= (state as u64) << bundle;

        Some(old_state)
    }
    // Read a component state by bitshifting
    pub fn get(&self, bundle: usize) -> Option<bool> {
        let borrowed = self.rows.borrow();
        let row = borrowed.get(bundle)?;
        Some(((*row >> bundle) & 1) == 1) 
    }
}
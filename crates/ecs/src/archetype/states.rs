use crate::Mask;
use std::cell::RefCell;

// Component state chunk that contains the component states for a bundle
#[derive(Clone, Copy, Debug)]
pub(crate) struct ComponentStateRow(Mask, Mask);

impl ComponentStateRow {
    // Create a chunk for newly added bundles using it's linked mask
    pub const fn new(mask: Mask) -> Self {
        Self(mask, mask)
    }

    // Check if a component was added during the current frame
    pub fn added(&self, offset: usize) -> bool {
        self.1.get(offset)
    }
    // Check if a component was mutated since the start of the current frame
    pub fn mutated(&self, offset: usize) -> bool {
        self.0.get(offset)
    }

    // Modify the two states
    pub fn update(&mut self, f: impl FnOnce(&mut Mask, &mut Mask)) {
        f(&mut self.0, &mut self.1);
    }
}

// Component states (their mutation state)
#[derive(Default)]
pub(crate) struct ComponentStateSet {
    rows: RefCell<Vec<ComponentStateRow>>,
}

impl ComponentStateSet {
    // Reset the component states to their default values
    pub fn reset(&self) {
        self.rows.borrow_mut().iter_mut().for_each(|row| {
            row.update(|a, b| {
                *a = Mask::zero();
                *b = Mask::zero();
            })
        });
    }

    // Add a new component states row
    pub fn push(&self, state: ComponentStateRow) {
        self.rows.borrow_mut().push(state);
    }

    // Remove a row and replace the empty spot with the last element in the set
    pub fn swap_remove(&self, bundle: usize) {
        self.rows.borrow_mut().swap_remove(bundle);
    }

    // Update the value of a row. This will return the old row state
    pub fn update(&self, bundle: usize, function: impl FnOnce(&mut Mask, &mut Mask)) -> Option<ComponentStateRow> {
        // Fetch the element
        let mut borrowed = self.rows.borrow_mut();
        let row = borrowed.get_mut(bundle)?;

        // Update
        let copy = *row;
        row.update(function);
        Some(copy)
    }

    // Reserve enough capacity to hold "additional" more rows
    pub fn reserve(&self, additional: usize) {
        self.rows.borrow_mut().reserve(additional);
    }

    // Get all the component states for a specific row
    pub fn get(&self, bundle: usize) -> Option<ComponentStateRow> {
        self.rows.borrow().get(bundle).cloned()
    }
}

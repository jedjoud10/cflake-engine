use crate::Mask;
use std::cell::RefCell;

// Component state chunk that contains the component states for a bundle
#[derive(Clone, Copy)]
pub(crate) struct ComponentStateRow(Mask, Mask);

impl ComponentStateRow {
    // Create a chunk for newly added bundles using it's linked mask
    pub const fn new(mask: Mask) -> Self {
        Self(mask, mask)
    }

    // Check if a component was added during the current frame
    pub fn added(&self, offset: usize) -> bool {
        (self.1 >> offset) == Mask::one()
    }
    // Check if a component was mutated since the start of the current frame
    pub fn mutated(&self, offset: usize) -> bool {
        (self.0 >> offset) == Mask::one()
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

    // Overwrite the state for a single row, returning the last row value
    pub fn overwrite(&self, bundle: usize, row: ComponentStateRow) -> Option<ComponentStateRow> {
        // Fetch the element
        let mut borrowed = self.rows.borrow_mut();
        let fetched = borrowed.get_mut(bundle)?;

        // Replace the element
        let old = std::mem::replace(fetched, row);
        Some(old)
    }

    // Update the value of a row
    pub fn update(&self, bundle: usize, function: impl FnOnce(&mut ComponentStateRow)) -> Option<()> {
        // Fetch the element
        let mut borrowed = self.rows.borrow_mut();
        let row = borrowed.get_mut(bundle)?;

        // Update
        function(row);
        Some(())
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

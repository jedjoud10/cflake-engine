use crate::{Component, StateColumn, UntypedVec};

// Typed component that will be converted to a Box<dyn UntypedColumn>
// This will not be stored within the archetype, but only to access internal component data
pub struct UntypedColumn {
    // Internal component data
    data: Box<dyn UntypedVec>,

    // Internal states stored in the column
    states: StateColumn,
}

impl UntypedColumn {
    // Create a new untyped column from a boxed untyped vec
    pub(crate) fn new(data: Box<dyn UntypedVec>) -> Self {
        Self {
            data,
            states: Default::default(),
        }
    }

    // Get the number of rows stored in this column
    pub fn len(&self) -> usize {
        assert_eq!(self.data.len(), self.states.len());
        self.data.len()
    }

    // Clear the column completely
    pub(crate) fn clear(&mut self) {
        self.data.clear();
        self.states.clear();
        assert_eq!(self.data.len(), self.states.len());
    }

    // Reserve more space to add more components
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
        self.states.reserve(additional);
    }

    // Shrink the memory allocation as much as possible
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
        self.states.shrink_to_fit();
    }

    // Get an immutable reference to the states
    pub fn states(&self) -> &StateColumn {
        &self.states
    }

    // Get a mutable reference to the states
    pub(crate) fn states_mut(&mut self) -> &mut StateColumn {
        &mut self.states
    }

    // Get an immutable reference to the components
    pub fn components(&self) -> &dyn UntypedVec {
        &*self.data
    }

    // Get a mutable reference to the components
    pub(crate) fn components_mut(&mut self) -> &mut dyn UntypedVec {
        &mut *self.data
    }

    // Try to cast the internally stored component vector to Vec<T> and return it as an immutable "typed column"
    pub fn as_<T: Component>(
        &self,
    ) -> Option<(&Vec<T>, &StateColumn)> {
        let vec = self.data.as_any().downcast_ref::<Vec<T>>()?;
        let states = &self.states;
        Some((vec, states))
    }

    // Try to cast the internally stored component vector to Vec<T> and return it as a mutable "typed column"
    pub(crate) fn as_mut_<T: Component>(
        &mut self,
    ) -> Option<(&mut Vec<T>, &mut StateColumn)> {
        let vec = self.data.as_any_mut().downcast_mut::<Vec<T>>()?;
        let states = &mut self.states;
        Some((vec, states))
    }

    pub fn clone_default(&self) -> Self {
        Self {
            data: self.data.clone_default(),
            states: Default::default(),
        }
    }
}

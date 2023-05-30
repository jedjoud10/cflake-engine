use crate::{Component, StateColumn, UntypedVec};

// Untyped column that contains the untyped vec (for components) and the delta frame and delta tick states
pub struct UntypedColumn {
    // Internal component data
    data: Box<dyn UntypedVec>,

    // Internal delta frame states stored in the column
    delta_frame_states: StateColumn,

    // Internal delta tick states stored in the column
    delta_tick_states: StateColumn, 
}

impl UntypedColumn {
    // Create a new untyped column from a boxed untyped vec
    pub(crate) fn new(data: Box<dyn UntypedVec>) -> Self {
        Self {
            data,
            delta_frame_states: Default::default(),
            delta_tick_states: Default::default(),
        }
    }

    // Get the number of rows stored in this column
    pub fn len(&self) -> usize {
        assert_eq!(self.data.len(), self.delta_frame_states.len());
        assert_eq!(self.data.len(), self.delta_tick_states.len());
        self.data.len()
    }

    // Clear the column completely
    pub(crate) fn clear(&mut self) {
        self.data.clear();
        self.delta_frame_states.clear();
        self.delta_tick_states.clear();
        assert_eq!(self.data.len(), self.delta_frame_states.len());
        assert_eq!(self.data.len(), self.delta_tick_states.len());
    }

    // Reserve more space to add more components
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
        self.delta_frame_states.reserve(additional);
        self.delta_tick_states.reserve(additional);
    }

    // Shrink the memory allocation as much as possible
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
        self.delta_frame_states.shrink_to_fit();
        self.delta_tick_states.shrink_to_fit();
    }

    // Get an immutable reference to the per-frame states
    pub fn delta_frame_states(&self) -> &StateColumn {
        &self.delta_frame_states
    }

    // Get a mutable reference to the per-frame states
    pub(crate) fn delta_frame_states_mut(&mut self) -> &mut StateColumn {
        &mut self.delta_frame_states
    }

    // Get an immutable reference to the per-tick states
    pub fn delta_tick_states(&self) -> &StateColumn {
        &self.delta_tick_states
    }

    // Get a mutable reference to the per-tick states
    pub(crate) fn delta_tick_states_mut(&mut self) -> &mut StateColumn {
        &mut self.delta_tick_states 
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
    pub fn as_<T: Component>(&self) -> Option<(&Vec<T>, &StateColumn, &StateColumn)> {
        let vec = self.data.as_any().downcast_ref::<Vec<T>>()?;
        let delta_frame_states = &self.delta_frame_states;
        let delta_tick_states = &self.delta_tick_states;
        Some((vec, delta_frame_states, delta_tick_states))
    }

    // Try to cast the internally stored component vector to Vec<T> and return it as a mutable "typed column"
    pub(crate) fn as_mut_<T: Component>(&mut self) -> Option<(&mut Vec<T>, &mut StateColumn, &mut StateColumn)> {
        let vec = self.data.as_any_mut().downcast_mut::<Vec<T>>()?;
        let delta_frame_states = &mut self.delta_frame_states;
        let delta_tick_states = &mut self.delta_tick_states;
        Some((vec, delta_frame_states, delta_tick_states))
    }

    pub fn clone_default(&self) -> Self {
        Self {
            data: self.data.clone_default(),
            delta_frame_states: Default::default(),
            delta_tick_states: Default::default(),
        }
    }
}

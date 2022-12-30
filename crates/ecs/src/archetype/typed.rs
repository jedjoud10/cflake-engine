use std::{mem::MaybeUninit, any::Any};
use crate::{Component, StateColumn, UntypedColumn, StateFlags};

// Typed component that will be converted to a Box<dyn UntypedColumn>
// This will not be stored within the archetype, but only to access internal component data 
pub struct Column<T: Component> {
    // Internal component data
    data: Vec<T>,

    // Internal states stored in the column
    states: StateColumn,
}

impl<T: Component> Column<T> {
    // Create a new typed column (so we can box it)
    pub(crate) fn new() -> Self {
        Self {
            data: Default::default(),
            states: Default::default(),
        }
    }

    // Get the number of rows stored in this column
    pub fn len(&self) -> usize {
        assert_eq!(self.data.len(), self.states.len());
        self.data.len()
    }

    // Clear the column completely
    pub fn clear(&mut self) {
        self.data.clear();
        self.states.clear();
    }

    // Swap remove a component at an index
    pub fn swap_remove(&mut self, index: usize) -> (T, StateFlags) {
        let flags = self.states.swap_remove(index).unwrap();
        let component = self.data.swap_remove(index);
        (component, flags)
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

    // Get an immutable reference to the vector
    pub fn components(&self) -> &Vec<T> {
        &self.data
    }
    
    // Get a mutable reference to the component vector
    pub fn components_mut(&mut self) -> &mut Vec<T> {
        &mut self.data
    }

    // Get the components as an immutable slice
    pub fn as_slice(&self) -> &[T] {
        self.data.as_slice()
    }

    // Get the components as a mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.data.as_mut_slice()
    }

    // Get a component item immutably from the typed column
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    // Get a component item mutably from the column without activating a mutation change state
    pub fn get_mut_silent(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }
    
    // Get a component item mutably from the column that will also be tracked
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.states.update(index, |flags| flags.modified = true);
        self.data.get_mut(index)
    }
}

impl<T: Component> UntypedColumn for Column<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn swap_remove(&mut self, index: usize) {
        Column::swap_remove(self, index);
    }

    fn swap_remove_move(
        &mut self,
        index: usize,
        other: &mut dyn UntypedColumn,
    ) {
        let (removed, flags) = Column::swap_remove(self, index);
        let other =
            other.as_any_mut().downcast_mut::<Self>().unwrap();

        other.data.push(removed);
        other.states.extend_with_flags(1, flags);
    }

    fn swap_remove_move_any_vec(
        &mut self,
        index: usize,
        vec: &mut dyn Any,
    ) {
        let (removed, _) = Column::swap_remove(self, index);
        let other = vec.downcast_mut::<Vec<T>>().unwrap();
        other.push(removed);
    }

    fn reserve(&mut self, additional: usize) {
        Column::reserve(self, additional);
    }

    fn shrink_to_fit(&mut self) {
        Column::shrink_to_fit(self);
    }

    fn states(&self) -> &StateColumn {
        &self.states
    }

    fn states_mut(&mut self) -> &mut StateColumn {
        &mut self.states   
    }

    fn components(&self) -> &dyn crate::UntypedVec {
        &self.data
    }

    fn components_mut(&mut self) -> &mut dyn crate::UntypedVec {
        &mut self.data
    }

    fn clear(&mut self) {
        Column::clear(self)
    }

    fn len(&self) -> usize {
        Column::len(self)
    }

    fn clone_default(&self) -> Box<dyn UntypedColumn> {
        Box::new(Self {
            data: Default::default(),
            states: Default::default(),
        })
    }
}
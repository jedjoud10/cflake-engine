use utils::{UntypedPtr, UntypedMutPtr};

use crate::{Component, StateColumn, UntypedVec};
use std::{any::{Any, TypeId}, mem::MaybeUninit};

// A component storage that is implemented for Vec<T>
// This type makes a lot of assumption about the parameters
// that are given to it, so it is only used internally
pub trait UntypedColumn {
    // Runtime dynamic conversions
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    // Remove a component from the storage, and move the last element into it's place instead
    fn swap_remove(&mut self, index: usize);

    // Remove a component from the storage, and insert the return value into another untyped column
    // This assumes that "other" is of the same type as Self
    fn swap_remove_move(
        &mut self,
        index: usize,
        other: &mut dyn UntypedColumn,
    );

    // Remove a component from the storage, and insert the return value into a vector of type Vec<T> where T: component
    // This assumes that "other" is of the type Vec<T>
    // Used internally when removing and dissociating removed components
    fn swap_remove_move_any_vec(
        &mut self,
        index: usize,
        vec: &mut dyn Any,
    );

    // Reserve some allocation space for the storage
    fn reserve(&mut self, additional: usize);

    // Shrink the memory allocation so it takes less space
    fn shrink_to_fit(&mut self);

    // Get the internally stored states immutably
    fn states(&self) -> &StateColumn;

    // Get the internally stored states mutably
    fn states_mut(&mut self) -> &mut StateColumn;

    // Get the internally stored components immutably
    fn components(&self) -> &dyn UntypedVec;

    // Get the internally stored componnets mutably
    fn components_mut(&mut self) -> &mut dyn UntypedVec;

    // Clear the column
    fn clear(&mut self);

    // Get the length of the vector
    fn len(&self) -> usize;

    // This will create an empty ComponentColumn vector using another one (to keep the trait object safe)
    fn clone_default(&self) -> Box<dyn UntypedColumn>;
}
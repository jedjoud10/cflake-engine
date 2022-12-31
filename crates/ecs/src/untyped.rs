use utils::{UntypedPtr, UntypedMutPtr, UntypedVec};
use crate::{Component, StateColumn};
use std::{any::{Any, TypeId}, mem::MaybeUninit};

// A component storage that is implemented for Vec<T>
// This type makes a lot of assumption about the parameters
// that are given to it, so it is only used internally
pub trait UntypedVec {
    // Runtime dynamic conversions
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    // Remove a value from the vec, and move the last element into it's place instead
    fn swap_remove(&mut self, index: usize);

    // Remove a value from the vec, and insert the return value into another component vec
    // This assumes that "other" is of the same type as Self
    fn swap_remove_move(
        &mut self,
        index: usize,
        other: &mut dyn UntypedVec,
    );

    // Reserve some allocation space for the storage
    fn reserve(&mut self, additional: usize);

    // Shrink the memory allocation so it takes less space
    fn shrink_to_fit(&mut self);

    // Get the length of the vector
    fn len(&self) -> usize;

    // This will create an empty ComponentColumn vector using another one (to keep the trait object safe)
    fn clone_default(&self) -> Box<dyn UntypedColumn>;
}
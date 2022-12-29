use crate::Component;
use std::{any::Any, mem::MaybeUninit};

// A component storage that is implemented for Vec<T>
// This type makes a lot of assumption about the parameters
// that are given to it, so it is only used internally
pub trait ComponentColumn {
    // Runtime dynamic conversions
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    // Remove a component from the storage, and move the last element into it's place instead
    fn swap_remove(&mut self, index: usize);

    // Remove a component from the storage, and insert the return value into another component storage
    // This assumes that "other" is of the same type as Self and that the old value is not zombified
    fn swap_remove_move(
        &mut self,
        index: usize,
        other: &mut dyn ComponentColumn,
    );

    // Zombify a component from the storage, and insert the return value into another component storage
    // This will uninitialize the memory of the old value (zombie data) until next frame, then it will be
    // completely removed from the vector
    // This assumes that "other" is of the same type as Self and that the old value is not zombified
    unsafe fn swap_remove_move_zombify(
        &mut self,
        index: usize,
        other: &mut dyn ComponentColumn, 
    );

    // Reserve some allocation space for the storage
    fn reserve(&mut self, additional: usize);

    // Shrink the memory allocation so it takes less space
    fn shrink_to_fit(&mut self);

    // Get the length of the vector
    fn len(&self) -> usize;

    // This will create an empty ComponentColumn vector using another one (to keep the trait object safe)
    fn clone_default(&self) -> Box<dyn ComponentColumn>;
}

pub type Column<T> = Vec<MaybeUninit<T>>;

impl<T: Component> ComponentColumn for Column<T> {
    // Convert to immutably Any trait object
    fn as_any(&self) -> &dyn Any {
        self
    }

    // Convert to mutable Any trait object
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    // Calls to Vec::swap_remove but typeless
    fn swap_remove(&mut self, index: usize) {
        self.swap_remove(index);
    }

    // Calls to Vec::swap_remove, and inserts the result into another storage
    fn swap_remove_move(
        &mut self,
        index: usize,
        other: &mut dyn ComponentColumn,
    ) {
        let removed = Vec::swap_remove(self, index);
        let other =
            other.as_any_mut().downcast_mut::<Self>().unwrap();
        other.push(removed);
    }

    // Moves the at "index" to the "other" column, then zombifies the old spot
    unsafe fn swap_remove_move_zombify(
        &mut self,
        index: usize,
        other: &mut dyn ComponentColumn, 
    ) {
        let null = MaybeUninit::<T>::uninit();
        let removed = std::mem::replace(&mut self[index], null);
        
        let other =
            other.as_any_mut().downcast_mut::<Self>().unwrap();
        other.push(removed);
    }

    // Reserve more memory to fit "additional" more elements
    fn reserve(&mut self, additional: usize) {
        Vec::reserve(self, additional);
    }

    // Shrink the memory allocation
    fn shrink_to_fit(&mut self) {
        Vec::shrink_to_fit(self);
    }

    // Get the length of the storage
    fn len(&self) -> usize {
        Vec::len(self)
    }

    // Create a new boxed component column based on the default state of Vec<T>
    fn clone_default(&self) -> Box<dyn ComponentColumn> {
        Box::new(Vec::<MaybeUninit<T>>::new())
    }
}

use std::any::Any;

// A component storage that is implemented for Vec<T>
// TODO: Convert to struct instead?
pub trait UntypedVec {
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
        other: &mut dyn UntypedVec,
    );

    // Reserve some allocation space for the storage
    fn reserve(&mut self, additional: usize);

    // Shrink the memory allocation so it takes less space
    fn shrink_to_fit(&mut self);

    // Clear the vector completely
    fn clear(&mut self);

    // Get the length of the vector
    fn len(&self) -> usize;

    // This will create an empty UntypedVec using another one (to keep the trait object safe)
    fn clone_default(&self) -> Box<dyn UntypedVec>;
}

impl<T: 'static> UntypedVec for Vec<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn swap_remove(&mut self, index: usize) {
        Vec::<T>::swap_remove(self, index);
    }

    fn swap_remove_move(
        &mut self,
        index: usize,
        other: &mut dyn UntypedVec,
    ) {
        let removed = Vec::swap_remove(self, index);
        let other =
            other.as_any_mut().downcast_mut::<Self>().unwrap();

        other.push(removed);
    }

    fn reserve(&mut self, additional: usize) {
        Vec::reserve(self, additional)
    }

    fn shrink_to_fit(&mut self) {
        Vec::shrink_to_fit(self)
    }

    fn clear(&mut self) {
        Vec::clear(self);
    }

    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn clone_default(&self) -> Box<dyn UntypedVec> {
        Box::new(Vec::<T>::new())
    }
}

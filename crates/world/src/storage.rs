use std::{cell::{UnsafeCell, RefCell, Cell}, marker::PhantomData, mem::{ManuallyDrop, MaybeUninit}, rc::Rc, ptr::NonNull, ops::Deref, num::NonZeroU32};
use crate::{Resource, World, FromWorld};
use crate as world;

// A FIFO queue that doesn't take a mutable reference to itself
// Only used internally for the InnerStorage that will be shared around
struct Queue<T>(RefCell<Vec<T>>);

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> Queue<T> {
    // Push a new value to the end of the queue
    fn push(&self, value: T) {
        self.0.borrow_mut().push(value);
    }

    // Try to get the last value that is stored within the queue
    fn pop(&self) -> Option<T> {
        self.0.borrow_mut().pop()
    }
}

// This trait will implemented for all InnerStorage<T> types
// This will automatically be called at the very very end of each frame, automatically, to cleanse all the storages of any dangling values that might be stored within them
// It's like a garbage collector, but type safe and in rust
trait Cleanse {
    fn remove_dangling(&mut self);
}

// A slot contains some raw data and the amount of references (aka strong handles) exist for it
type Slot<T> = (UnsafeCell<MaybeUninit<T>>, Cell<(u32, bool)>);

// This the inner storage that will be shared around using the Rc and RefCell
pub struct InnerStorage<T> {
    slots: RefCell<Vec<Slot<T>>>,
    empty: Queue<usize>,
}

impl<T> Cleanse for InnerStorage<T> {
    fn remove_dangling(&mut self) {
        // Just drop 
    }
}

// A storage set descriptor contains multiple reference to multiple InnerStorage<T>s 
// This allows us to cleanse each storage of any dangling values automatically using a specific system
#[derive(Resource)]
pub struct StorageSetDescriptor {
    storages: Vec<Rc<dyn Cleanse>>,
} 

// This is the main system that will "cleanse" the stored storages
pub fn cleanse(world: &mut World) {

}

// A storage is a way to keep certain values stored in memory without dropping them
// When inserting a new value into a storage, we receive a Handle to that value
// Handles can be cloned around to be able to share values and save memory,
// though if the last handle of a specific value gets dropped, the value will also get dropped
pub struct Storage<T: 'static>(Rc<InnerStorage<T>>);

// Create a new default storage using the world
// This will also register the storage's cleanse function automatically
impl<T> FromWorld for Storage<T> {
    fn from_world(world: &mut world::World) -> Self {
        let rc = Rc::new(InnerStorage { slots: Default::default(), empty: Default::default() });
        let descriptor = world.entry::<StorageSetDescriptor>().or_insert_with(|_| StorageSetDescriptor { storages: Default::default() });
        descriptor.storages.push(rc.clone() as Rc<dyn Cleanse>);
        Self(rc)
    }
}

// Storages automatically get inserted into the world when we try to access them, so we must write that custom logic when implementing the resource trait
// Also, we update the main StorageDescriptors resource, since we must update this storage when we can
impl<T: 'static> Resource for Storage<T> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn fetch_ptr(world: &mut world::World) -> Result<std::ptr::NonNull<Self>, world::ResourceError> where Self: Sized {
        let res = world.entry::<Self>().or_insert_from_world();
        Ok(NonNull::new(res as *mut Self).unwrap())
    }
}

impl<T: 'static> Storage<T> {
    // Insert a new value into the storage, and return it's tracker handle
    // This value will stay stored within the storage as long as it has a single valid handle
    pub fn insert(&mut self, value: T) -> Handle<T> {
        // Check if we have any free slots that we can use
        let idx = if let Some(idx) = self.0.empty.pop() {
            // Overwrite the existing cell, but make sure we don't cause a second drop by any chance
            let mut slots = self.0.slots.borrow_mut();
            let (cell, counter) = &mut slots[idx];

            // If we already dropped the value, don't drop it again
            let old = unsafe { &mut *cell.get() }; 
            if counter.get().is_some() {
                // Overwrite the old value with the new values without dropping them again
                unsafe {
                    std::ptr::write(old.as_mut_ptr(), value);
                }
            } else {
                // Overwrite the value (which will also drop the old value)
                *old = MaybeUninit::new(value);
                counter.set(Some(NonZeroU32::new(1).unwrap()));
            }
            
            // Return the index of the new value
            idx
        } else {
            // We have no free empty slots, so we must make a new one
            let mut slots = self.0.slots.borrow_mut();
            let idx = slots.len();

            // Create the values that we will insert
            let cell = UnsafeCell::new(MaybeUninit::new(value));
            let count = Cell::new(Some(NonZeroU32::new(1).unwrap()));
            
            // Insert the slot and return it's index
            slots.push((cell, count));
            idx
        };

        // Create the handle object
        Handle {
            _phantom: Default::default(),
            inner: self.0.clone(),
            idx,
        }
    }

    // Get an immutable reference to a value stored within the stored using it's handle
    // The handle must outlive the returned reference, since dropping the handle before then might cause UB
    pub fn get(&self, handle: &Handle<T>) -> &T {
        let slots = self.0.slots.borrow();
        let ptr = unsafe { &*slots[handle.idx].0.get() };
        unsafe { ptr.assume_init_ref() }
    }

    // Get an mutable reference to a value stored within the stored using it's handle
    // The handle must outlive the returned reference, since dropping the handle before then might cause UB
    pub fn get_mut(&mut self, handle: &Handle<T>) -> &mut T {
        let slots = self.0.slots.borrow_mut();
        let ptr = unsafe { &mut *slots[handle.idx].0.get() };
        unsafe { ptr.assume_init_mut() }
    }
}

// A handle is what keeps the values within Storage<T> alive
// If we drop the last Handle<T> to a value T, then the value will be removed from the storage and dropped
pub struct Handle<T: 'static> {
    _phantom: PhantomData<*mut T>,
    inner: Rc<InnerStorage<T>>,
    idx: usize,
}

impl<T: 'static> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
}

impl<T: 'static> Eq for Handle<T> {
}

impl<T: 'static> Handle<T> {
    // Get the current reference count for this handle
    // This tells us how many valid handles exist for the current value (including the current handle)
    pub fn count(&self) -> u32 {
        let slots = self.inner.slots.borrow();
        slots[self.idx].1.get().unwrap().get()
    }

    // This will manually incremememnt the underlying reference counter
    // This is pretty unsafe, since it will mess up how the handles work internally
    // This will return the new reference counter value after we incremented it
    pub unsafe fn increment_count(&self) -> NonZeroU32 {
        // Calculate new counter value (this can never be None, since it is impossible to become 0)
        let value = NonZeroU32::new_unchecked(self.count().saturating_add(1)); 

        // Set the new counter value
        let slots = self.inner.slots.borrow();
        slots[self.idx].1.set(Some(value));
        value
    }

    // This will manually decrement the underlying reference counter
    // This is pretty unsafe, since it will mess up how the handles work internally
    // This will return the new reference counter alue after we decremented it
    // If the counter reaches 0 references (meaning that the value should be dropped), this will return None
    pub unsafe fn decrement_count(&self) -> Option<NonZeroU32> {
        // We don't unwrap since we want it to become None if it reaches 0
        let value = NonZeroU32::new(self.count().saturating_sub(1));

        // Set the new counter value
        let slots = self.inner.slots.borrow();
        slots[self.idx].1.set(value);
        value
    }
}

// Cloning the handle will increase the reference count of that handle
impl<T: 'static> Clone for Handle<T> {
    fn clone(&self) -> Self {
        unsafe { self.increment_count(); }        
        Self {
            _phantom: Default::default(),
            inner: self.inner.clone(),
            idx: self.idx,
        }
    }
} 

// Dropping the handle will decrease the reference count of that handle
// If we drop the last valid handle, then the stored value will get dropped
impl<T: 'static> Drop for Handle<T> {
    fn drop(&mut self) {        
        // The cleansing system will automatically get rid of danlging values for us
        unsafe { self.decrement_count(); }
    }
}
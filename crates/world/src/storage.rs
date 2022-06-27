use std::{cell::{UnsafeCell, RefCell}, marker::PhantomData, mem::{ManuallyDrop, MaybeUninit}, rc::Rc, ptr::NonNull, ops::Deref};
use crate::{Resource, World};
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
 

// This the inner storage that will be shared around using the Rc and RefCell
pub struct InnerStorage<T> {
    slots: RefCell<Vec<UnsafeCell<(MaybeUninit<T>, u32)>>>,
    empty: Queue<usize>,
}

impl<T> Cleanse for InnerStorage<T> {
    fn remove_dangling(&mut self) {
        todo!()
    }
}

// A storage set descriptor contains multiple reference to multiple InnerStorage<T>s 
// This allows us to cleanse each storage of any dangling values automatically using a specific system
#[derive(Resource)]
pub struct StorageSetDescriptor {
    storages: Vec<Rc<dyn Cleanse>>,
} 

// A storage is a way to keep certain values stored in memory without dropping them
// When inserting a new value into a storage, we receive a Handle to that value
// Handles can be cloned around to be able to share values and save memory,
// though if the last handle of a specific value gets dropped, the value will also get dropped
pub struct Storage<T: 'static>(Rc<InnerStorage<T>>);

// Create a new default storage using the world
// This will also register the storage's cleanse function automatically
impl<T> From<&mut World> for Storage<T> {
    fn from(world: &mut world::World) -> Self {
        let rc = Rc::new(InnerStorage { slots: Default::default(), empty: Default::default() });
        let descriptor = world.entry::<StorageSetDescriptor>().or_insert_with(|_| StorageSetDescriptor { storages: Default::default() });
        descriptor.storages.push(rc.clone() as Rc<dyn Cleanse>);
        Self(rc)
    }
}
/*
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
        if !world.contains::<Self>() {
            world.insert(Self::from_world());
        }

        let res = world.get_mut_unique::<Self>().unwrap();
        Ok(NonNull::new(res as *mut Self).unwrap())
    }
}

impl<T: 'static> Storage<T> {
    // Insert a new value into the storage, and return it's tracker handle
    // This value will stay stored within the storage as long as it has a single valid handle
    pub fn insert(&mut self, value: T) -> Handle<T> {
        let (ptr, idx) = if self.empty.borrow().is_empty() {
            // We have no free slots, we have to make a new slot and fill it in
            self.slots.push(UnsafeCell::new((value, 1)));

            // Return the pointer and value index
            let index = self.slots.len() - 1; 
            let ptr = self.slots[index].get();
            (ptr, index)
        } else {
            // We have at least one free slot, so we must overwrite the values stored within it (without dropping them!)
            let empty = self.empty.borrow_mut().pop().unwrap();
            let cell = &mut self.slots[empty];

            // Overwrite the values. This is actually safe since the empty slot was dropped beforehand
            unsafe {
                std::ptr::write(cell.get(), (value, 1));
            }

            // Return the pointer and value index
            (cell.get(), empty)
        };

        // Create the new handle object
        Handle { _phantom: Default::default(), ptr, idx, empty: self.empty.clone() }
    }

    // Get an immutable reference to a value stored within the stored using it's handle
    // The handle must outlive the returned reference, since dropping the handle before then might cause UB
    pub fn get<'s, 'c, 'h: 'c>(&'s self, handle: &'h Handle<T>) -> &'c T {
        &unsafe { &*self.slots[handle.idx as usize].get() }.0
    }

    // Get an mutable reference to a value stored within the stored using it's handle
    // The handle must outlive the returned reference, since dropping the handle before then might cause UB
    pub fn get_mut<'s, 'c, 'h: 'c>(&'s mut self, handle: &'h Handle<T>) -> &'c mut T {
        &mut unsafe { &mut *self.slots[handle.idx as usize].get() }.0
    }
}

// A handle is what keeps the values within Storage<T> alive
// If we drop the last Handle<T> to a value T, then the value will be removed from the storage and dropped
pub struct Handle<T: 'static> {
    _phantom: PhantomData<*mut T>,
    storage: Rc<InnerStorage<T>>,
    empty: Rc<RefCell<Vec<usize>>>,
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
        unsafe {
            &*self.ptr
        }.1
    }

    // This will manually incremememnt the underlying reference counter
    // This is pretty unsafe, since it will mess up how the handles work
    pub unsafe fn increment_count(&self) -> u32 {
        let (_, counter) = &mut *self.ptr;
        *counter += 1;
        *counter
    }

    // This will manually decrement the underlying reference counter
    // This is pretty unsafe, since it will mess up how the handles work
    pub unsafe fn decrement_count(&self) -> u32 {
        let (_, counter) = &mut *self.ptr;
        *counter -= 1;
        *counter
    }
}

// Cloning the handle will increase the reference count of that handle
impl<T: 'static> Clone for Handle<T> {
    fn clone(&self) -> Self {
        unsafe { self.increment_count(); }        
        Self { 
            _phantom: self._phantom.clone(),
            ptr: self.ptr.clone(),
            empty: self.empty.clone(),
            idx: self.idx.clone()
        }
    }
} 

// Dropping the handle will decrease the reference count of that handle
// If we drop the last valid handle, then the stored value will get dropped
impl<T: 'static> Drop for Handle<T> {
    fn drop(&mut self) {
        // Drop if this is the last handle
        if unsafe { self.decrement_count() } == 0 {
            // Convert the slot into an empty slot
            let mut empty = self.empty.borrow_mut();
            empty.push(self.idx);
            
            // Drop the value
            unsafe {
                std::ptr::drop_in_place(self.ptr);
            }
        }
    }
}
*/
use std::{sync::{atomic::{AtomicPtr, Ordering::Relaxed}, Arc}, ptr::null_mut};

// Some sort of external ID that we can generate on other threads, and that will be passed to the main thread whenever we actually want to generate the ID
pub trait ExternalID<T>
    where Self: Sized
{
    // Get the pointer
    fn ptr(&self) -> &Arc<AtomicPtr<T>>;
    // Update the value of the pointer. WE MUST DO THIS ON THE MAIN THREAD
    fn set(self, id: &T) {
        let ptr = self.ptr();
        ptr.store((id as *const T) as *mut T, Relaxed);
    }
    // Get the number of references that the internal Arc contains
    fn ref_count(&self) -> usize { Arc::strong_count(self.ptr()) }
    // Check if the pointer simply null
    fn is_null(&self) -> bool { self.ptr().load(Relaxed).is_null() }
    // Invalidate the pointer if it is not null
    fn invalidate(self) {
        if !self.is_null() {
            // Invalidation
            self.ptr().store(null_mut(), Relaxed);
        } else {
            // Uh oh
            panic!()
        }
    }
    // Check if the pointer is null, and try to get it. WE MUST DO THIS ON THE MAIN THREAD
    fn try_get(&self) -> Option<T> {
        if self.is_null() {
            None
        } else {  
            Some(unsafe { self.ptr().load(Relaxed).read() })
        }
    }
}
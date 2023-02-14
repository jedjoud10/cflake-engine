use std::{mem::MaybeUninit, cell::UnsafeCell, sync::atomic::{AtomicUsize, Ordering}};
use parking_lot::Mutex;
type Cell<T> = UnsafeCell<MaybeUninit<T>>;

struct Chunk<T> {
    data: Box<[Cell<T>; 16]>
}

// An atomic vec can contain one or more values for the lifetime of the Vec itself
pub struct AtomicVec<T> {
    chunks: Mutex<Vec<Chunk<T>>>,
    length: AtomicUsize,
}

impl<T> Default for AtomicVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for AtomicVec<T> {
    fn drop(&mut self) {
        let mut lock = self.chunks.lock();
        let length = self.length.load(Ordering::Relaxed);
        
        let mut index = 0;
        for chunk in lock.iter_mut() {
            for cell in chunk.data.iter_mut() {
                let value = cell.get_mut();
                
                if index < length {
                    unsafe {
                        value.assume_init_drop();
                    }
                }

                index += 1;
            }
        }
    }
}

impl<T> AtomicVec<T> {
    // Create a new atomic vector that can store values forever
    pub fn new() -> Self {
        Self {
            chunks: Mutex::new(Vec::new()),
            length: AtomicUsize::new(0)
        }
    }

    // Add a new value to the atomic vector and store it forever
    pub fn push(&self, value: T) {
        let len = self.length.fetch_add(1, Ordering::Relaxed) + 1;
        let mut locked = self.chunks.lock();
        let chunk = len / 16;
        let location = len % 16;

        // Make sure the chunk exists
        if chunk <= locked.len() {
            locked.push(Chunk {
                data: unsafe {
                    let data = MaybeUnint::<[]>
                }
                //data: Box::new([UnsafeCell::new(MaybeUninit::uninit()); 16]),
            })
        }

        let chunk = locked.get_mut(chunk).unwrap();
        let cell = &mut chunk.data[location];
        *cell = UnsafeCell::new(MaybeUninit::new(value));
    }

    // Gets a value immutably using an index, returns None if it doesn't exist 
    pub fn get(&self, index: usize) -> Option<&T> {
        let locked = self.chunks.lock();
        let chunk = index / 16;
        let location = index % 16;
        let chunk = locked.get(chunk)?;
        let data =  chunk.data.get(location)?;
        let ptr = data.get() as *const T;
    
        Some(unsafe {
            &*ptr
        })
    }
}
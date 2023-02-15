use std::{mem::MaybeUninit, cell::UnsafeCell, sync::atomic::{AtomicUsize, Ordering}};
use parking_lot::{Mutex, MutexGuard};

type Cell<T> = MaybeUninit<UnsafeCell<T>>;
struct Chunk<T> {
    data: Box<[Cell<T>; 16]>
}

// An atomic vec can contain one or more values for the lifetime of the Vec itself
// TODO: Handle multiple locks at the same time
pub struct AtomicVec<T> {
    chunks: Mutex<Vec<Chunk<T>>>,
    length: AtomicUsize,
}

impl<T> Default for AtomicVec<T> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<T> Sync for AtomicVec<T> {}
unsafe impl<T> Send for AtomicVec<T> {}


impl<T> Drop for AtomicVec<T> {
    fn drop(&mut self) {
        let mut lock = self.chunks.lock();
        let length = self.length.load(Ordering::Relaxed);
        
        let mut index = 0;
        for chunk in lock.iter_mut() {
            for cell in chunk.data.iter_mut() {
                if index < length {
                    unsafe {
                        cell.assume_init_drop();
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
        let index = self.length.fetch_add(1, Ordering::Relaxed);
        let mut locked = self.chunks.lock();
        let chunk = index / 16;
        let location = index % 16;

        // Make sure the chunk exists
        if chunk <= locked.len() {
            locked.push(Chunk {
                data: unsafe {
                    let data: [Cell<T>; 16] = MaybeUninit::uninit().assume_init();
                    Box::new(data)
                }
            })
        }

        let chunk = locked.get_mut(chunk).unwrap();
        let cell = &mut chunk.data[location];
        *cell = MaybeUninit::new(UnsafeCell::new(value));
    }

    // Gets a value immutably using an index, returns None if it doesn't exist 
    pub fn get(&self, index: usize) -> Option<&T> {
        let length = self.length.load(Ordering::Relaxed);
        
        if index >= length {
            return None;
        }
        
        let locked = self.chunks.lock();
        let chunk = index / 16;
        let location = index % 16;
        let chunk = locked.get(chunk).unwrap();
        let data =  &chunk.data[location];
        unsafe {
            let ptr = data.assume_init_read().get() as *const T;
            Some(&*ptr)
        }
    }

    // Check if the atomic vec is currently locked
    pub fn is_locked(&self) -> bool {
        self.chunks.is_locked()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        Iter {
            len: self.length.load(Ordering::Relaxed),
            guard: self.chunks.lock(),
            index: 0,
        }
    }
}

pub struct Iter<'a, T> {
    guard: MutexGuard<'a, Vec<Chunk<T>>>,
    index: usize,
    len: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len {
            return None;
        }

        let chunk = self.index / 16;
        let location = self.index % 16;

        let chunk = &self.guard[chunk];
        let data = &chunk.data[location];
        self.index += 1;
        
        Some(unsafe {
            let ptr = data.assume_init_ref().get();
            &*ptr
        })
    }
}
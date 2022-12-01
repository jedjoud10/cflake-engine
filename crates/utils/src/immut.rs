use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicUsize, Ordering},
};

use parking_lot::{Mutex, RwLock};

// Number of elements per page
pub const ELEMENTS_PER_PAGE: usize = 32;

// A page of an immutable vector
pub struct Page<T> {
    chunk: Box<[UnsafeCell<MaybeUninit<T>>; ELEMENTS_PER_PAGE]>,
}

unsafe impl<T> Sync for Page<T> {}
unsafe impl<T> Send for Page<T> {}

// An immutable vector that can grow and shrink in size
// However, it cannot mutate any of it's components
pub struct ImmutableVec<T> {
    pages: RwLock<Vec<Page<T>>>,
    index: AtomicUsize,
}

impl<T> ImmutableVec<T> {
    // Create a new immutable vector
    pub fn new() -> Self {
        Self {
            pages: RwLock::new(Vec::new()),
            index: AtomicUsize::new(0),
        }
    }

    // Add multiple items from an iterator
    pub fn extend(&self, iterator: impl Iterator<Item = T>) {
        for element in iterator {
            self.push(element);
        }
    }

    // Add a new element
    pub fn push(&self, value: T) {
        let len = self.index.fetch_add(1, Ordering::Relaxed);

        // Calculate location and page before we add the index
        let location = len % ELEMENTS_PER_PAGE;

        // Calculate pages amnd their differences
        let old_page = len / ELEMENTS_PER_PAGE;
        let new_page = (len + 1) / ELEMENTS_PER_PAGE;

        // Check if we need to add new pages
        if new_page >= self.pages.read().len() {
            let additional = (new_page - old_page) + 1;
            let iter = (0..additional).into_iter().map(|_| {
                // Create the array (MaybeUninit::uninit_array())
                let array = unsafe {
                    MaybeUninit::<
                        [UnsafeCell<MaybeUninit<T>>;
                            ELEMENTS_PER_PAGE],
                    >::uninit()
                    .assume_init()
                };

                // Create the page struct
                Page {
                    chunk: Box::new(array),
                }
            });
            self.pages.write().extend(iter)
        }

        // Lock and fetch the page
        let locked = self.pages.read();
        let new_page = (len + 1) / ELEMENTS_PER_PAGE;
        let page = &locked[new_page];

        // __Please__ don't look at this
        let ptr = page.chunk[location].get();
        unsafe { std::ptr::write(ptr, MaybeUninit::new(value)) }
    }

    /*
    // Remove the last element
    pub fn pop(&self) -> Option<T> {
        let index = self.index.load(Ordering::Relaxed).checked_sub(1)?;
        self.index.fetch_sub(1, Ordering::Relaxed);
        let location = index % ELEMENTS_PER_PAGE;
        let mut locked = self.pages.write();
        let last_page = locked.last_mut()?;

        // __Please__ don't look at this
        let ptr = &mut last_page.chunk[location];
        let uninit = ptr.get_mut();

        // Read from the pointer and return the old value
        let inner = unsafe {
            std::ptr::replace(
                uninit,
                MaybeUninit::uninit()
            ).assume_init()
        };
        Some(inner)
    }
    */

    // Get the element at index i
    pub fn get(&self, i: usize) -> Option<&T> {
        let len = self.index.load(Ordering::Relaxed);

        if i >= len {
            // Return early if the index does not exist
            return None;
        } else {
            // Calculate indices
            let location = i % ELEMENTS_PER_PAGE;
            let page = i / ELEMENTS_PER_PAGE;

            // Fetch page
            let lock = self.pages.read();
            let page = &lock[page];

            // Don't look at this
            let ptr = page.chunk[location].get();
            let outer = unsafe { (&*ptr).assume_init_ref() };
            Some(outer)
        }
    }

    // Get the last element in this immutable vector
    pub fn last(&self) -> Option<&T> {
        self.get(self.index.load(Ordering::Relaxed).checked_sub(1)?)
    }

    // Get the number of elements
    pub fn len(&self) -> usize {
        self.index.load(Ordering::Relaxed)
    }
}

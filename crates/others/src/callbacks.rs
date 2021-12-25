use lazy_static::lazy_static;
use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::HashMap,
    sync::{atomic::AtomicU64, Mutex},
    thread::LocalKey,
};

// A ref callback, always ran at the end of the current system frame
pub struct RefCallback<T, D> {
    pub callback: Box<dyn Fn(&T, D)>,
}
// A mutable callback that mutates that value passed. Always ran at the end of the world thread frame
pub struct MutCallback<T, D> {
    pub callback: Box<dyn Fn(&mut T, D)>,
}
// An owned callback, always ran at the end of the current system frame
pub struct OwnedCallback<T, D> {
    pub callback: Box<dyn Fn(T, D)>,
}
// A callback that just executes, but it doesn't have any data to pass around
pub struct NullCallback<D> {
    pub callback: Box<dyn Fn(D)>,
}

impl<T, D> RefCallback<T, D> {
    pub fn new<F>(c: F) -> Self
    where
        F: Fn(&T, D) + 'static,
    {
        let callback = Box::new(c);
        Self { callback }
    }
}

impl<T, D> MutCallback<T, D> {
    pub fn new<F>(c: F) -> Self
    where
        F: Fn(&mut T, D) + 'static,
    {
        let callback = Box::new(c);
        Self { callback }
    }
}

impl<T, D> OwnedCallback<T, D> {
    pub fn new<F>(c: F) -> Self
    where
        F: Fn(T, D) + 'static,
    {
        let callback = Box::new(c);
        Self { callback }
    }
}

impl<D> NullCallback<D> {
    pub fn new<F>(c: F) -> Self
    where
        F: Fn(D) + 'static,
    {
        let callback = Box::new(c);
        Self { callback }
    }
}

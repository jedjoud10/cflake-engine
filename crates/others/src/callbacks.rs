use lazy_static::lazy_static;
use std::{
    borrow::BorrowMut,
    cell::RefCell,
    collections::HashMap,
    sync::{atomic::AtomicU64, Mutex},
    thread::LocalKey,
};

// A ref callback, always ran at the end of the current system frame
pub struct RefCallback<T> {
    pub callback: Box<dyn Fn(&T)>,
}
// A mutable callback that mutates that value passed. Always ran at the end of the world thread frame
pub struct MutCallback<T> {
    pub callback: Box<dyn Fn(&mut T)>,
}
// An owned callback, always ran at the end of the current system frame
pub struct OwnedCallback<T> {
    pub callback: Box<dyn Fn(T)>,
}
// A callback that just executes, but it doesn't have any data to pass around
pub struct NullCallback {
    pub callback: Box<dyn Fn()>,
}

impl<T> RefCallback<T> {
    pub fn new<F>(c: F) -> Self
    where
        F: Fn(&T) + 'static,
    {
        let callback = Box::new(c);
        Self { callback }
    }
}

impl<T> MutCallback<T> {
    pub fn new<F>(c: F) -> Self
    where
        F: Fn(&mut T) + 'static,
    {
        let callback = Box::new(c);
        Self { callback }
    }
}

impl<T> OwnedCallback<T> {
    pub fn new<F>(c: F) -> Self
    where
        F: Fn(T) + 'static,
    {
        let callback = Box::new(c);
        Self { callback }
    }
}

impl NullCallback {
    pub fn new<F>(c: F) -> Self
    where
        F: Fn() + 'static,
    {
        let callback = Box::new(c);
        Self { callback }
    }
}

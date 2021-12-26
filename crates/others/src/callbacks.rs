// A ref callback, always ran at the end of the current system frame
pub struct RefCallback<T> {
    pub callback: Box<dyn FnOnce(&T)>,
}
// A mutable callback that mutates that value passed. Always ran at the end of the world thread frame
pub struct MutCallback<T> {
    pub callback: Box<dyn FnOnce(&mut T)>,
}
// An owned callback, always ran at the end of the current system frame
pub struct OwnedCallback<T> {
    pub callback: Box<dyn FnOnce(T)>,
}
// A callback that just executes, but it doesn't have any data to pass around
pub struct NullCallback {
    pub callback: Box<dyn FnOnce()>,
}

impl<T> RefCallback<T> {
    pub fn new<F>(c: F) -> Self
    where
        F: FnOnce(&T) + 'static,
    {
        let callback = Box::new(c);
        Self { callback }
    }
}

impl<T> MutCallback<T> {
    pub fn new<F>(c: F) -> Self
    where
        F: FnOnce(&mut T) + 'static,
    {
        let callback = Box::new(c);
        Self { callback }
    }
}

impl<T> OwnedCallback<T> {
    pub fn new<F>(c: F) -> Self
    where
        F: FnOnce(T) + 'static,
    {
        let callback = Box::new(c);
        Self { callback }
    }
}

impl NullCallback {
    pub fn new<F>(c: F) -> Self
    where
        F: FnOnce() + 'static,
    {
        let callback = Box::new(c);
        Self { callback }
    }
}

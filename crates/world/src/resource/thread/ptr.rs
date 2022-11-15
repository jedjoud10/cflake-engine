use std::any::TypeId;

// Unsafe ref pointer that we can send to other threads
pub struct SendPtr<T>(*const T);
unsafe impl<T> Sync for SendPtr<T> {}
unsafe impl<T> Send for SendPtr<T> {}

impl<T> Clone for SendPtr<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}
impl<T> Copy for SendPtr<T> {}

impl<T: 'static> Into<*const T> for SendPtr<T> {
    fn into(self) -> *const T {
        self.0
    }
}

impl<T: 'static> From<*const T> for SendPtr<T> {
    fn from(value: *const T) -> Self {
        Self(value)
    }
}

// Unsafe mut pointer that we can send to other threads
pub struct SendMutPtr<T>(*mut T);
unsafe impl<T> Sync for SendMutPtr<T> {}
unsafe impl<T> Send for SendMutPtr<T> {}

impl<T> Clone for SendMutPtr<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}
impl<T> Copy for SendMutPtr<T> {}

impl<T: 'static> Into<*mut T> for SendMutPtr<T> {
    fn into(self) -> *mut T {
        self.0
    }
}

impl<T: 'static> From<*mut T> for SendMutPtr<T> {
    fn from(value: *mut T) -> Self {
        Self(value)
    }
}

// Unsafe untyped pointer that we can send to other threads
pub struct UntypedPtr(*const (), TypeId);

impl<T: 'static> Into<*const T> for UntypedPtr {
    fn into(self) -> *const T {
        self.0 as *const T
    }
}

impl<T: 'static> From<*const T> for UntypedPtr {
    fn from(ptr: *const T) -> Self {
        Self(ptr as *const (), TypeId::of::<T>())
    }
}

unsafe impl Sync for UntypedPtr {}
unsafe impl Send for UntypedPtr {}

// Unsafe untyped mut pointer that we can send to other threads
pub struct UntypedMutPtr(*mut (), TypeId);

impl<T: 'static> Into<*mut T> for UntypedMutPtr {
    fn into(self) -> *mut T {
        self.0 as *mut T
    }
}

impl<T: 'static> From<*mut T> for UntypedMutPtr {
    fn from(ptr: *mut T) -> Self {
        Self(ptr as *mut (), TypeId::of::<T>())
    }
}

unsafe impl Sync for UntypedMutPtr {}
unsafe impl Send for UntypedMutPtr {}

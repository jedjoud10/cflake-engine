// I don't want to use Box<T>
pub struct Stored<T> where T: Sized {
    pub ptr: *const T
}

impl<T> Stored<T> {
    pub fn new(reference: &T) -> Self {
        Self {
            ptr: reference as *const T,
        }
    }
}

impl<T> std::ops::Deref for Stored<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target { unsafe { &*self.ptr } }
}

pub struct StoredMut<T> where T: Sized {
    pub ptr_mut: *mut T
}

impl<T> StoredMut<T> {
    pub fn new_mut(reference_mut: &mut T) -> Self {
        Self {
            ptr_mut: reference_mut as *mut T,
        }
    }
}

impl<T> std::ops::Deref for StoredMut<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target { unsafe { &*self.ptr_mut } }
}

impl<T> std::ops::DerefMut for StoredMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target { unsafe { &mut *self.ptr_mut } }
}
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
use crate::ComponentInternal;

// I don't want to use Box<T>
pub struct Stored<T>
where
    T: Sized,
{
    pub ptr: *const T,
}

impl<T> Stored<T> {
    pub fn new(reference: &T) -> Self {
        Self { ptr: reference as *const T }
    }
}

impl Stored<Box<dyn ComponentInternal + Send + Sync>> {
    // Cast the stored self pointer to the component T
    pub fn cast<U: ComponentInternal + Send + Sync + 'static>(&self) -> Stored<U> {
        let boxed = unsafe { &*self.ptr };
        let t = boxed.as_ref().as_any().downcast_ref::<U>().unwrap();
        Stored::new(t)
    }
}
 
impl<T> std::ops::Deref for Stored<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

pub struct StoredMut<T>
where
    T: Sized,
{
    pub ptr_mut: *mut T,
}

impl<T> StoredMut<T> {
    pub fn new_mut(reference_mut: &mut T) -> Self {
        Self { ptr_mut: reference_mut as *mut T }
    }
}

impl StoredMut<Box<dyn ComponentInternal + Send + Sync>> {
    // Cast the stored self pointer to the component T
    pub fn cast<U: ComponentInternal + Send + Sync + 'static>(&self) -> StoredMut<U> {
        let boxed = unsafe { &mut *self.ptr_mut };
        let t = boxed.as_mut().as_any_mut().downcast_mut::<U>().unwrap();
        StoredMut::new_mut(t)
    }
}

impl<T> std::ops::Deref for StoredMut<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr_mut }
    }
}

impl<T> std::ops::DerefMut for StoredMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr_mut }
    }
}

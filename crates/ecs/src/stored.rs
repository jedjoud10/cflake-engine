use std::marker::PhantomData;

use crate::{Component, Entity};

// I don't want to use Box<T>
pub struct Stored<'a, T>
where
    T: Sized + Component,
{
    pub global_id: &'a usize,
    pub ptr: *const T,
    marker: PhantomData<&'a T>,
}

impl<'a, T> Stored<'a, T> 
where
    T: Sized + Component {
    pub fn new(component: &T, global_id: &'a usize) -> Self {
        Self {
            global_id,
            ptr: component as *const T,
            marker: PhantomData,
        }
    }
}
/*
impl Stored<Box<dyn ComponentInternal + Send + Sync>> {
    // Cast the stored self pointer to the component T
    pub fn cast<U: ComponentInternal + Send + Sync + 'static>(&self) -> Stored<U> {
        let boxed = unsafe { &*self.ptr };
        let t = boxed.as_ref().as_any().downcast_ref::<U>().unwrap();
        Stored::new(t, self.global_id)
    }
}
*/

impl<'a, T> Stored<'a, T> where T: Sized + Component + 'static {
    pub fn get(&self, entity: &'a Entity) -> &'a T {
        let component_ptr = unsafe { &*self.ptr };
        component_ptr
    }
}

pub struct StoredMut<'a, T>
where
    T: Sized + Component,
{
    pub global_id: &'a usize,
    pub ptr: *mut T,
    marker: PhantomData<&'a mut T>,
}

impl<'a, T> StoredMut<'a, T> 
where
    T: Sized + Component {
    pub fn new(component: &mut T, global_id: &'a usize) -> Self {
        Self {
            global_id,
            ptr: component as *mut T,
            marker: PhantomData,
        }
    }
}
/*
impl Stored<Box<dyn ComponentInternal + Send + Sync>> {
    // Cast the stored self pointer to the component T
    pub fn cast<U: ComponentInternal + Send + Sync + 'static>(&self) -> Stored<U> {
        let boxed = unsafe { &*self.ptr };
        let t = boxed.as_ref().as_any().downcast_ref::<U>().unwrap();
        Stored::new(t, self.global_id)
    }
}
*/

impl<'a, T> StoredMut<'a, T> 
where 
    T: Sized + Component + 'static {
    
    pub fn get_mut(&self, entity: &'a Entity) -> &'a mut T {
        let component_ptr = unsafe { &mut *self.ptr };
        component_ptr
    }
}
use std::marker::PhantomData;

use crate::Texture;

// This struct allows us to send variables to the currently used pipeline shader
// We use a lifetime to guarantee that the objects that we use live longer than the pipeline itself
pub struct Bindings<'a> {
    pub(crate) _phantom: PhantomData<&'a ()>,
}

impl<'a> Bindings<'a> { 
    pub fn test<T>(&'a self, value: &'a T) {
    }

    pub fn set_texture<T: Texture>(&'a self,  texture: &T) {

    }
}
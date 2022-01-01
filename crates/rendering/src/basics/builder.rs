use std::marker::PhantomData;

use crate::Texture;

// A simple builder that can be used to create Pipeline Objects
pub struct Builder<T> {
    phantom: PhantomData<T>
}

// This will create the AsyncPipelineObject and return it, while also send it to the render thread
impl<T> Builder<T> {
    // Create the AsyncPipelineObject
    pub fn build(self) -> AsyncPipelineObject<T> {
        todo!();
    }
}

impl Builder<Texture> {

}
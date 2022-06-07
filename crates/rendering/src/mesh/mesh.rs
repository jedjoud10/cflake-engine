use world::resources::Handle;

use crate::material::Material;
use super::SubMesh;

// A surface is just a simple submesh that is linked with a handle
pub struct Surface<M: Material>(Handle<SubMesh>, Handle<M>);

impl<M: Material> Surface<M> {
    // Create a new surface with the valid handles
    pub fn new(submesh: Handle<SubMesh>, material: Handle<M>) -> Self {
        Self(submesh, material)
    }
    
    // Get the submesh handle
    pub fn submesh(&self) -> Handle<SubMesh> { self.0.clone() }

    // Get the material handle
    pub fn material(&self) -> Handle<M> { self.1.clone() }
}
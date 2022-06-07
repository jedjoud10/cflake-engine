use world::resources::Handle;
use crate::material::Material;
use super::SubMesh;

// A surface is just a simple submesh that is linked with a handle
pub struct Surface(Handle<SubMesh>, Handle<Material>);

impl Surface {
    // Create a new surface with the valid handles
    pub fn new(submesh: Handle<SubMesh>, material: Handle<Material>) -> Self {
        Self(submesh, material)
    }
    
    // Get the submesh handle
    pub fn submesh(&self) -> Handle<SubMesh> { self.0.clone() }

    // Get the material handle
    pub fn material(&self) -> Handle<Material> { self.1.clone() }
}
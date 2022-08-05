
use math::{UvSphere, Cuboid, IcoSphere};

use crate::context::Context;

use super::{Mesh, MeshImportSettings};

// A primitive generator that we can use to generate procedural shapes at runtime
pub trait PrimitiveGenerator {
    fn generate(self, ctx: &mut Context, settings: MeshImportSettings) -> Mesh;
}

impl PrimitiveGenerator for Cuboid {
    // Generate a cuboid mesh
    fn generate(self, ctx: &mut Context, settings: MeshImportSettings) -> Mesh {
        todo!()
    }
}

impl PrimitiveGenerator for UvSphere {
    // Generate a UV sphere mesh
    fn generate(self, ctx: &mut Context, settings: MeshImportSettings) -> Mesh {
        todo!()
    }
}

impl PrimitiveGenerator for IcoSphere {
    // Generate an IcoSphere mesh
    fn generate(self, ctx: &mut Context, settings: MeshImportSettings) -> Mesh {
        todo!()
    }
}
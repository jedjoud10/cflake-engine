
use super::{Mesh};

// A primitive generator that we can use to generate procedural cubes and spheres at runtime
pub trait PrimitiveGenerator<'ctx> where Self: 'ctx {
    fn generate(self) -> Mesh;
}

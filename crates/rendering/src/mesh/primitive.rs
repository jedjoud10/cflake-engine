use crate::context::Context;
use super::{MeshImportSettings, Mesh};

// A primitive generator that we can use to generate procedural cubes and spheres at runtime
pub trait PrimitiveGenerator<'ctx> where Self: 'ctx {
    fn generate(self) -> Mesh;
}

// Settings used when generating a cuboid
pub struct PrimitiveCuboidSettings<'a> {
    pub geom: math::Cuboid,
    pub settings: MeshImportSettings,
    pub ctx: &'a mut Context,
}

// Settings used when generating a uv sphere
pub struct PrimitiveUvSphereSettings<'a> {
    pub geom: math::Sphere,
    pub horizontal_subdivisions: u32,
    pub vertical_subdivions: u32,
    pub settings: MeshImportSettings,
    pub ctx: &'a mut Context,
}

// Settings used when generting an ico sphere
pub struct PrimitiveIcoSphereSettings<'a> {
    pub geom: math::Sphere,
    pub subdivisions: u32,
    pub settings: MeshImportSettings,
    pub ctx: &'a mut Context,
}


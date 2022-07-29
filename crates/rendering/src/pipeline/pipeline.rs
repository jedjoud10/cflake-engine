use crate::{
    canvas::{PrimitiveMode, RasterSettings, Canvas},
    context::{Context, Window},
    mesh::{Mesh, Surface},
    prelude::{Shader, Uniforms},
    material::{AlbedoMap, Material},
    scene::{Camera, Directional, Renderer, SceneSettings}, buffer::ElementBuffer,
};
use assets::{Assets, Asset};
use ecs::Scene;
use math::{Location, Rotation};
use std::{any::type_name, marker::PhantomData};
use world::{Handle, Read, Resource, Storage, World};

pub struct PipelineInitData<'a> {
    pub shaders: &'a mut Storage<Shader>,
    pub assets: &'a mut Assets,
}

// Marker type that tells us we registered a specific generic pipeline
pub struct PipeId<P: Pipeline>(pub(crate) PhantomData<P>);

// Pipeline trait that will be boxed and stored from within the world
// Pipelines are user defined to allow the user to write their own logic
pub trait Pipeline: 'static {
    fn init(init: &mut PipelineInitData, ctx: &mut Context) -> Self where Self: Sized;
    fn render(&self, world: &mut World);
}

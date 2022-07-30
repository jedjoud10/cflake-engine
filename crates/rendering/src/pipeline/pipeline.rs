use crate::{
    canvas::{PrimitiveMode, RasterSettings, Canvas},
    context::{Context, Window},
    mesh::{Mesh, Surface},
    prelude::{Shader, Uniforms},
    material::{AlbedoMap, Material},
    scene::{Camera, DirectionalLight, Renderer}, buffer::ElementBuffer,
};
use assets::{Assets, Asset};
use ecs::Scene;
use math::{Location, Rotation};
use std::{any::type_name, marker::PhantomData};
use world::{Handle, Read, Resource, Storage, World};

// Marker type that tells us we registered a specific generic pipeline
pub struct PipeId<P: Pipeline>(pub(crate) PhantomData<P>);

// Pipeline trait that will be boxed and stored from within the world
// Pipelines are user defined to allow the user to write their own logic
pub trait Pipeline: 'static {
    fn render(&self, world: &mut World);
}

// This is a custom pipeline creator that will be able to instantiate a specific pipeline
pub trait CreatePipeline<'a>: Pipeline + Sized {
    type Args: 'a + Sized;
    fn init(ctx: &mut Context, args: &mut Self::Args) -> Self;
}
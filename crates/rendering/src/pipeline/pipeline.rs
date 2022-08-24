use crate::{context::Context, scene::RenderedFrameStats};

use std::marker::PhantomData;
use world::World;

// Marker type that tells us we registered a specific generic pipeline
pub struct PipeId<P: Pipeline>(pub(crate) PhantomData<P>);

impl<P: Pipeline> Clone for PipeId<P> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// Pipeline trait that will be boxed and stored from within the world
// Pipelines are user defined to allow the user to write their own logic
pub trait Pipeline: 'static {
    fn render(&self, world: &mut World, stats: &mut RenderedFrameStats);
}

// This is a custom pipeline creator that will be able to instantiate a specific pipeline
pub trait CreatePipeline<'a>: Pipeline + Sized {
    type Args: 'a + Sized;
    fn init(ctx: &mut Context, args: &mut Self::Args) -> Self;
}

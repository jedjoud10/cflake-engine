use std::marker::PhantomData;

use world::{World, Handle};

use crate::prelude::Shader;

use super::{Material, PropertyBlock, Standard};

// Statistics that tell us what exactly happened when we rendered the material surfaces through the pipeline
pub struct Stats {}

// A material pipeline contain the logic telling us how we should render and draw a specific material type
pub trait Pipeline: 'static {
    // Pepare the pipeline for rendering
    fn prepare(&self, world: &mut World) {}

    // Cull any surfaces if needed
    fn cull(&self, world: &mut World) {}

    // Render the materialized surface onto the screen
    fn render(&self, world: &mut World) -> Option<Stats>;

    // Called after we render
    fn cleanup(&self, world: &mut World) {}
}

// Custom renderers
pub trait PipelineRenderer<M: Material> {
    // Render the materialized surfaces
    fn render<'w>(&self, world: &'w mut World) -> Option<Stats> where M: PropertyBlock<'w>;
}


impl<R: PipelineRenderer<Standard> + 'static> Pipeline for R {
    fn render(&self, world: &mut World) -> Option<Stats> {
        <R as PipelineRenderer<Standard>>::render(&self, world)
    }
} 
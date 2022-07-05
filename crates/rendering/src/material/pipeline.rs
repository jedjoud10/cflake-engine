use std::marker::PhantomData;
use world::{World, Handle};
use crate::prelude::Shader;
use super::{Material, PropertyBlock, Standard};

// Statistics that tell us what exactly happened when we rendered the material surfaces through the pipeline
pub struct Stats {}

// A material pipeline contain the logic telling us how we should render and draw a specific material type
// For now, the only rendering pipeline type is batch rendering 
pub trait MaterialRenderer: 'static {
    fn render(&self, world: &mut World) -> Option<Stats>;
}

impl<F: Fn(&mut World) -> Option<Stats> + 'static> MaterialRenderer for F {
    fn render(&self, world: &mut World) -> Option<Stats> {
        todo!()
    }
}

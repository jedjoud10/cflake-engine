use std::{marker::PhantomData, rc::Rc};
use world::{World, Handle};
use crate::prelude::Shader;
use super::{Material, PropertyBlock, Standard};

// Statistics that tell us what exactly happened when we rendered the material surfaces through the pipeline
pub struct Stats {}

// A material renderer is responsible for rendering and drawing surfaces of a specific material onto the screen
// For now, material renderers are implemented as functions that can be called back
pub trait Pipeline: 'static {
    // Create a new pipeline from a shader
    fn new(shader: Handle<Shader>) where Self: Sized;

    // Fetch the shader handle from the pipeline
    fn shader(&self) -> Handle<Shader>;

    // Cull all the surfaces that we will render
    fn cull(self, world: &mut World);

    // Render all the materialized surfaces
    fn render(self, world: &mut World);

    // Post-render method
    fn cleanup(self, world: &mut World) {}
}

// The default pipeline that uses one shader pass to render everything
// TODO: Find better name
pub struct DefaultPipeline<M: Material + for<'a> PropertyBlock<'a>> {
    shader: Handle<Shader>,
    _phantom: PhantomData<M>,
}

impl<M: Material + for<'a> PropertyBlock<'a>> Pipeline for DefaultPipeline<M> {
    fn new(shader: Handle<Shader>) where Self: Sized {
        Self {
            shader,
            _phantom: Default::default(),
        }
    }

    fn shader(&self) -> Handle<Shader> {
        todo!()
    }

    fn cull(self, world: &mut World) {
        todo!()
    }

    fn render(self, world: &mut World) {
        todo!()
    }
}

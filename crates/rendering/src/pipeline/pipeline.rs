use std::marker::PhantomData;
use world::World;
use crate::Material;

// A material ID is used to make sure the user has initialized the proper material pipeline
pub struct MaterialId<M: Material>(pub(crate) PhantomData<M>);

impl<M: Material> Clone for MaterialId<M> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

// A material pipeline will be responsible for rendering surface and
// entities that correspond to a specific material type.
pub struct Pipeline<M: Material> {
    pipeline: graphics::GraphicsPipeline,
    id: MaterialId<M>,
    _phantom: PhantomData<M>,
}

impl<M: Material> Pipeline<M> {
    // Create a new material pipeline
    pub fn new() -> Self {
        todo!()
    }
}

// This trait will be implemented for Pipeline<T> to allow for dynamic dispatch
pub trait DynamicPipeline {
    // Get the inner graphics pipeline
    fn graphical(&self) -> &graphics::GraphicsPipeline;

    // Get the inner graphics shader

    // Render all surfaces that use the material of this pipeline
    fn render(&self, world: &mut World);
}

impl<M: Material> DynamicPipeline for Pipeline<M> {
    fn graphical(&self) -> &graphics::GraphicsPipeline {
        &self.pipeline
    }

    fn render(&self, world: &mut World) {
        super::render_surfaces::<M>(world);
    }
}
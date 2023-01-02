use std::marker::PhantomData;
use assets::Assets;
use graphics::{Shader, GraphicsPipeline, RenderPass};
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
    pipeline: GraphicsPipeline,
    shader: Shader,
    _phantom: PhantomData<M>,
}

impl<M: Material> Pipeline<M> {
    // Create a new material pipeline for the given material
    // This will load the shader, and create the graphics pipeline
    pub fn new(assets: &Assets, render_pass: RenderPass) -> Self {
        let vertex = M::vertex(assets);
        let fragment = M::fragment(assets);
        let shader = Shader::new(vertex, fragment);

        // Create the graphics pipeline
        let pipeline = unsafe {
            GraphicsPipeline::new(
                M::depth_config(),
                M::stencil_config(),
                M::blend_config(),
                M::primitive_mode(),
                render_pass,
                shader.clone()
            )
        };

        Self {
            pipeline,
            shader,
            _phantom: PhantomData,
        }
    }

    // Get the material ID of the pipeline
    pub fn id(&self) -> MaterialId<M> {
        MaterialId(PhantomData)
    }
    
    // Get the material pipeline shader
    pub fn shader(&self) -> &Shader {
        &self.shader
    }
}

// This trait will be implemented for Pipeline<T> to allow for dynamic dispatch
pub trait DynamicPipeline {
    // Get the inner graphics pipeline
    fn graphical(&self) -> &GraphicsPipeline;

    // Get the inner graphics shader

    // Render all surfaces that use the material of this pipeline
    fn render(&self, world: &mut World);
}

impl<M: Material> DynamicPipeline for Pipeline<M> {
    fn graphical(&self) -> &GraphicsPipeline {
        &self.pipeline
    }

    fn render(&self, world: &mut World) {
        super::render_surfaces::<M>(world);
    }
}
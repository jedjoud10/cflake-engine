use crate::{Material, SwapchainFormat, ForwardRendererRenderPass};
use assets::Assets;
use graphics::{Graphics, GraphicsPipeline, RenderPass, Shader, ActiveRenderPass, VertexConfig, PipelineInitializationError};
use std::marker::PhantomData;
use world::World;

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
    pub fn new(
        graphics: &Graphics,
        assets: &Assets,
        render_pass: &ForwardRendererRenderPass,
    ) -> Result<Self, PipelineInitializationError> {
        let vertex = M::vertex(graphics, assets);
        let fragment = M::fragment(graphics, assets);
        let shader = Shader::new(vertex, fragment);
        
        // Fetch the correct vertex config based on the material
        let required = M::attributes();
        let untyped = crate::untyped_attributes_from_enabled_attributes(required);
        let vertex_config = VertexConfig {
            attributes: untyped.iter().map(|u| u.attribute).collect(),
            bindings: untyped.iter().map(|u| u.binding).collect(),
        };

        // Create the graphics pipeline
        let pipeline = GraphicsPipeline::new(
            graphics,
            M::depth_config(),
            M::stencil_config(),
            M::blend_config(),
            vertex_config,
            M::primitive(),
            M::binding_config(),
            &render_pass,
            shader.clone(),
        )?;

        Ok(Self {
            pipeline,
            shader,
            _phantom: PhantomData,
        })
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
    // Get the inner graphics pipeline immutably
    fn graphical(&self) -> &GraphicsPipeline;

    // Get the inner graphics pipeline mutably
    fn graphical_mut(&mut self) -> &mut GraphicsPipeline;

    // Render all surfaces that use the material of this pipeline
    fn render(&self,
        world: &World,
        render_pass: &mut ActiveRenderPass<SwapchainFormat, ()>
    );
}

impl<M: Material> DynamicPipeline for Pipeline<M> {
    fn graphical(&self) -> &GraphicsPipeline {
        &self.pipeline
    }

    fn graphical_mut(&mut self) -> &mut GraphicsPipeline {
        &mut self.pipeline
    }

    fn render(&self, world: &World, render_pass: &mut ActiveRenderPass<SwapchainFormat, ()>) {
        super::render_surfaces::<M>(world, &self.pipeline, render_pass);
    }
}

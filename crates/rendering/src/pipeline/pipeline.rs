use crate::{Material, ForwardRendererRenderPass, MeshAttribute, attributes::{TexCoord, Tangent, Normal, Position}, EnabledMeshAttributes};
use assets::Assets;
use graphics::{Graphics, GraphicsPipeline, RenderPass, Shader, VertexConfig, PipelineInitializationError, BindingConfig, VertexInput, SwapchainFormat};
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
    pipeline: GraphicsPipeline<SwapchainFormat, ()>,
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
        // Load the vertex and fragment modules, and create the shader
        let vertex = M::vertex(graphics, assets);
        let fragment = M::fragment(graphics, assets);
        let shader = Shader::new(&vertex,& fragment);
        
        // Fetch the correct vertex config based on the material
        let vertex_config = crate::attributes::enabled_to_vertex_config(
            M::attributes()
        );

        // Create the graphics pipeline
        let pipeline = GraphicsPipeline::new(
            graphics,
            M::depth_config(),
            M::stencil_config(),
            vertex_config,
            M::primitive_config(),
            BindingConfig {},
            &shader
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
    fn graphical(&self) -> &GraphicsPipeline<SwapchainFormat, ()>;

    // Get the inner graphics pipeline mutably
    fn graphical_mut(&mut self) -> &mut GraphicsPipeline<SwapchainFormat, ()>;

    // Render all surfaces that use the material of this pipeline
    fn render(&self,
        world: &World,
    );
}

impl<M: Material> DynamicPipeline for Pipeline<M> {
    fn graphical(&self) -> &GraphicsPipeline<SwapchainFormat, ()> {
        &self.pipeline
    }

    fn graphical_mut(&mut self) -> &mut GraphicsPipeline<SwapchainFormat, ()> {
        &mut self.pipeline
    }

    fn render(&self, world: &World) {
        //super::render_surfaces::<M>(world, &self.pipeline, render_pass);
    }
}

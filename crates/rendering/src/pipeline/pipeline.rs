use crate::{
    attributes::{
        Normal, Position, Tangent, TexCoord,
        MAX_MESH_VERTEX_ATTRIBUTES,
    },
    DefaultMaterialResources, EnabledMeshAttributes,
    ForwardRendererRenderPass, Material, Mesh, MeshAttribute,
};
use assets::Assets;
use graphics::{
    ActiveRenderPass, Graphics, GraphicsPipeline,
    PipelineInitializationError, RenderPass, Shader, SwapchainFormat,
    VertexConfig, VertexInput, Depth, DepthConfig, CompareFunction,
};
use std::marker::PhantomData;
use utils::Storage;
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
    pipeline: GraphicsPipeline<SwapchainFormat, Depth<f32>>,
    shader: Shader,
    _phantom: PhantomData<M>,
}

impl<M: Material> Pipeline<M> {
    // Create a new material pipeline for the given material
    // This will load the shader, and create the graphics pipeline
    pub fn new(
        graphics: &Graphics,
        assets: &Assets,
    ) -> Result<Self, PipelineInitializationError> {
        // Load the vertex and fragment modules, and create the shader
        let vertex = M::vertex(graphics, assets);
        let fragment = M::fragment(graphics, assets);
        let shader = Shader::new(graphics, &vertex, &fragment);

        // Fetch the correct vertex config based on the material
        let vertex_config =
            crate::attributes::enabled_to_vertex_config(
                M::attributes(),
            );

        // We must always have a depth config
        let depth_config = M::depth_config().unwrap_or(DepthConfig {
            compare: CompareFunction::Always,
            write_enabled: true,
            depth_bias_constant: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        });

        // Create the graphics pipeline
        let pipeline = GraphicsPipeline::new(
            graphics,
            Some(depth_config),
            M::stencil_config(),
            vertex_config,
            M::primitive_config(),
            &shader,
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
    // Render all surfaces that use the material of this pipeline
    fn render<'r>(
        &'r self,
        world: &'r World,
        meshes: &'r Storage<Mesh>,
        default: &'r DefaultMaterialResources,
        render_pass: &mut ActiveRenderPass<
            'r,
            '_,
            SwapchainFormat,
            Depth<f32>,
        >,
    );
}

impl<M: Material> DynamicPipeline for Pipeline<M> {
    fn render<'r>(
        &'r self,
        world: &'r World,
        meshes: &'r Storage<Mesh>,
        default: &'r DefaultMaterialResources,
        render_pass: &mut ActiveRenderPass<
            'r,
            '_,
            SwapchainFormat,
            Depth<f32>,
        >,
    ) {
        super::render_surfaces::<M>(
            world,
            meshes,
            &self.pipeline,
            default,
            render_pass,
        );
    }
}

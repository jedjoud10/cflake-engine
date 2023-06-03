use crate::{
    ActiveShadowRenderPass,
    DefaultMaterialResources, Material, SceneColorLayout, SceneDepthLayout, ShadowRenderPipeline, DeferredPass, Pass,
};

use assets::Assets;
use graphics::{
    CompareFunction, DepthConfig, Graphics, PipelineInitializationError, RenderPipeline, Shader, ActiveRenderPass,
};
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
// TODO: Reimplemt shadows
pub struct Pipeline<M: Material> {
    pipeline: RenderPipeline<SceneColorLayout, SceneDepthLayout>,
    shader: Shader,
    _phantom: PhantomData<M>,
}

impl<M: Material> Pipeline<M> {
    // Create a new material pipeline for the given material
    // This will load the shader, and create the graphics pipeline
    pub(crate) fn new(
        settings: M::Settings<'_>,
        graphics: &Graphics,
        assets: &Assets,
    ) -> Result<Self, PipelineInitializationError> {
        // Load the material's shader
        let shader = M::shader::<DeferredPass>(&settings, graphics, assets).unwrap();

        // Fetch the correct vertex config based on the material
        let vertex_config = crate::attributes::enabled_to_vertex_config(M::attributes::<DeferredPass>());

        // Default depth config for ALL materials
        let depth_config = DepthConfig {
            compare: CompareFunction::Less,
            write_enabled: true,
            depth_bias_constant: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0
        };
        
        // Create the graphics pipeline
        let pipeline = RenderPipeline::new(
            graphics,
            Some(depth_config),
            None,
            None,
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
pub trait DynPipeline {
    // Cull all surfaces before we render the scene
    fn cull<'r>(
        &'r self,
        world: &'r World,
        default: &mut DefaultMaterialResources<'r>,
    );

    // Render all surfaces using the main pass
    fn render<'r>(
        &'r self,
        world: &'r World,
        default: &mut DefaultMaterialResources<'r>,
        render_pass: &mut ActiveRenderPass::<'r, '_, SceneColorLayout, SceneDepthLayout>,
    );

    // Render all surfaces using the shadow pass
    fn render_shadow<'r>(
        &'r self,
        world: &'r World,
        default: &mut DefaultMaterialResources<'r>,
        render_pass: &mut ActiveRenderPass::<'r, '_, SceneColorLayout, SceneDepthLayout>,
    );
}

impl<M: Material> DynPipeline for Pipeline<M> {
    fn cull<'r>(
        &'r self,
        world: &'r World,
        default: &mut DefaultMaterialResources<'r>,
    ) {
        super::cull_surfaces::<M>(world, default);
    }

    fn render<'r>(
        &'r self,
        world: &'r World,
        default: &mut DefaultMaterialResources<'r>,
        render_pass: &mut ActiveRenderPass::<'r, '_, SceneColorLayout, SceneDepthLayout>,
    ) {
        super::render_surfaces::<DeferredPass, M>(world, &self.pipeline, default, render_pass);
    }

    fn render_shadow<'r>(
        &'r self,
        world: &'r World,
        default: &mut DefaultMaterialResources<'r>,
        render_pass: &mut ActiveRenderPass::<'r, '_, SceneColorLayout, SceneDepthLayout>,
    ) {
        super::render_surfaces::<DeferredPass, M>(world, &self.pipeline, default, render_pass);
    }
}

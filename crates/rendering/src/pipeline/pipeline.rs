use crate::{
    ActiveSceneRenderPass, ActiveShadowRenderPipeline,
    DefaultMaterialResources, Material,
    SceneColor, SceneDepth, ShadowRenderPipeline, ActiveShadowRenderPass, CastShadowsMode,
};

use assets::Assets;
use graphics::{
    CompareFunction, DepthConfig,
    Graphics, PipelineInitializationError, RenderPipeline, Shader,
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
pub struct Pipeline<M: Material> {
    pipeline: RenderPipeline<SceneColor, SceneDepth>,
    shadow_pipeline: Option<ShadowRenderPipeline>,
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
        let shader = M::shader(&settings, graphics, assets);

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
        let pipeline = RenderPipeline::new(
            graphics,
            Some(depth_config),
            M::stencil_config(),
            None,
            vertex_config,
            M::primitive_config(),
            &shader,
        )?;

        // Create a customized shadow render pipeline if requested
        let shadow_pipeline = if let CastShadowsMode::Enabled(Some(callback)) = M::casts_shadows() {
            let shader = callback(&settings, graphics, assets);
            Some(crate::create_shadow_render_pipeline(graphics, &shader))
        } else {
            None
        };

        Ok(Self {
            pipeline,
            shader,
            _phantom: PhantomData,
            shadow_pipeline,
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
    // Executed before we call the "render" event in batch
    fn render_shadows<'r>(
        &'r self,
        world: &'r World,
        default: &DefaultMaterialResources<'r>,
        active: &mut ActiveShadowRenderPass<'r, '_>,
        default_shadow_pipeline: &'r ShadowRenderPipeline,
        lightspace: vek::Mat4<f32>,
    );

    // Render all surfaces that use the material of this pipeline
    fn render<'r>(
        &'r self,
        world: &'r World,
        default: &mut DefaultMaterialResources<'r>,
        render_pass: &mut ActiveSceneRenderPass<'r, '_>,
    );
}

impl<M: Material> DynPipeline for Pipeline<M> {
    fn render_shadows<'r>(
        &'r self,
        world: &'r World,
        default: &DefaultMaterialResources<'r>,
        active: &mut ActiveShadowRenderPass<'r, '_>,
        default_shadow_pipeline: &'r ShadowRenderPipeline,
        lightspace: vek::Mat4<f32>,
    ) {
        let shadow_pipeline = self.shadow_pipeline.as_ref().unwrap_or(default_shadow_pipeline);
        super::render_shadows::<M>(world, default, active, shadow_pipeline, lightspace);
    }

    fn render<'r>(
        &'r self,
        world: &'r World,
        default: &mut DefaultMaterialResources<'r>,
        render_pass: &mut ActiveSceneRenderPass<'r, '_>,
    ) {
        super::cull_surfaces::<M>(world, default);

        super::render_surfaces::<M>(
            world,
            &self.pipeline,
            default,
            render_pass,
        );
    }
}

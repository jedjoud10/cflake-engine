use crate::{
    DefaultMaterialResources, Material, SceneColorLayout, SceneDepthLayout, DeferredPass, Pass, ShadowDepthLayout, ShadowPass, MeshAttributes,
};

use assets::Assets;
use graphics::{
    CompareFunction, DepthConfig, Graphics, PipelineInitializationError, RenderPipeline, Shader, ActiveRenderPass, PrimitiveConfig, WindingOrder, Face,
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
    // Base deferred render pass
    pipeline: RenderPipeline<SceneColorLayout, SceneDepthLayout>,
    shader: Shader,

    // Shadow render pass
    shadow_pipeline: Option<RenderPipeline<(), ShadowDepthLayout>>,
    shadow_shader: Option<Shader>,

    _phantom: PhantomData<M>,
}

// Default depth config for ALL materials
pub(crate) const DEPTH_CONFIG: DepthConfig = DepthConfig {
    compare: CompareFunction::Less,
    write_enabled: true,
    depth_bias_constant: 0,
    depth_bias_slope_scale: 0.0,
    depth_bias_clamp: 0.0
};

// Create a shadow render pipeline from a shadow shader
// This is called not only by the default shadowmap shader, but by materials that define their own shadow shader as well
pub(crate) fn create_shadow_render_pipeline(
    graphics: &Graphics,
    shader: &Shader,
    attributes: MeshAttributes,
) -> Result<RenderPipeline<(), ShadowDepthLayout>, PipelineInitializationError> {    
    RenderPipeline::<(), ShadowDepthLayout>::new(
        graphics,
        Some(DEPTH_CONFIG),
        None,
        None,
        crate::attributes::enabled_to_vertex_config(attributes),
        PrimitiveConfig::Triangles {
            winding_order: WindingOrder::Ccw,
            cull_face: Some(Face::Back),
            wireframe: false,
        },
        shader,
    )
}

fn create_deferred_render_pipeline<M: Material>(
    graphics: &Graphics,
    shader: &Shader
) -> Result<RenderPipeline<SceneColorLayout, SceneDepthLayout>, PipelineInitializationError> {
    RenderPipeline::new(
        graphics,
        Some(DEPTH_CONFIG),
        None,
        None,
        crate::attributes::enabled_to_vertex_config(M::attributes::<DeferredPass>()),
        M::primitive_config(),
        shader,
    )
}


impl<M: Material> Pipeline<M> {
    // Create a new material pipeline for the given material
    // This will load the shader, and create the graphics pipeline
    pub(crate) fn new(
        settings: M::Settings<'_>,
        graphics: &Graphics,
        assets: &Assets,
    ) -> Result<Self, PipelineInitializationError> {
        let shader = M::shader::<DeferredPass>(&settings, graphics, assets).unwrap();
        let pipeline = create_deferred_render_pipeline::<M>(graphics, &shader)?;

        let shadow_shader = M::shader::<ShadowPass>(&settings, graphics, assets);
        let shadow_attributes = M::attributes::<ShadowPass>();
        let shadow_pipeline = shadow_shader.as_ref().map(|x| create_shadow_render_pipeline(graphics, x, shadow_attributes));
        let shadow_pipeline = if let Some(pp) = shadow_pipeline { Some(pp?) } else { None };

        Ok(Self {
            pipeline,
            shader,
            _phantom: PhantomData,
            shadow_pipeline,
            shadow_shader,
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
        default: &DefaultMaterialResources<'r>,
    );

    // Render all surfaces using the main pass
    fn render<'r>(
        &'r self,
        world: &'r World,
        default: &DefaultMaterialResources<'r>,
        render_pass: &mut ActiveRenderPass::<'r, '_, SceneColorLayout, SceneDepthLayout>,
    );

    // Render all surfaces using the shadow pass
    fn render_shadow<'r>(
        &'r self,
        world: &'r World,
        default: &DefaultMaterialResources<'r>,
        render_pass: &mut ActiveRenderPass::<'r, '_, (), ShadowDepthLayout>,
    );
}

impl<M: Material> DynPipeline for Pipeline<M> {
    fn cull<'r>(
        &'r self,
        world: &'r World,
        default: &DefaultMaterialResources<'r>,
    ) {
        super::cull_surfaces::<M>(world, default);
    }

    fn render<'r>(
        &'r self,
        world: &'r World,
        default: &DefaultMaterialResources<'r>,
        render_pass: &mut ActiveRenderPass::<'r, '_, SceneColorLayout, SceneDepthLayout>,
    ) {
        super::render_surfaces::<DeferredPass, M>(world, &self.pipeline, default, render_pass);
    }

    fn render_shadow<'r>(
        &'r self,
        world: &'r World,
        default: &DefaultMaterialResources<'r>,
        render_pass: &mut ActiveRenderPass::<'r, '_, (), ShadowDepthLayout>,
    ) {
        if let Some(pipeline) = self.shadow_pipeline.as_ref() {
            super::render_surfaces::<ShadowPass, M>(world, pipeline, default, render_pass);
        }
    }
}

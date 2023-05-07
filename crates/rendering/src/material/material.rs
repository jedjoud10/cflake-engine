use crate::{
    ActiveScenePipeline, DefaultMaterialResources, MeshAttributes, RenderPath, Renderer, SceneColor,
};
use assets::Assets;

use ecs::QueryLayoutRef;
use graphics::{
    BindGroup, BlendConfig, CompareFunction, DepthConfig, Graphics, PrimitiveConfig, PushConstants,
    Shader, StencilConfig, WindingOrder,
};

use world::World;

// If materials can cast shadows onto other objects
// Also allows us to use a custom shadow shader
pub enum CastShadowsMode<M: Material> {
    Disabled,

    // If you *are* going to use a custom shadow shader you should note that only the position attribute is given
    // and other atrributes are NOT given
    // TODO: Find a way to let the user customize the shadow shader to their heart's extent
    Enabled(Option<Box<dyn FnOnce(&M::Settings<'_>, &Graphics, &Assets) -> Shader>>),
}

// A material is what defines the physical properties of surfaces whenever we draw them onto the screen
// Materials correspond to a specific WGPU render pipeline based on it's config parameters
pub trait Material: 'static + Sized + Sync + Send {
    // The resources that we need to fetch from the world to set the descriptor sets
    type Resources<'w>: 'w;
    type RenderPath: RenderPath;
    type Settings<'s>;
    type Query<'a>: ecs::QueryLayoutRef;

    // Create a shader for this material
    fn shader(settings: &Self::Settings<'_>, graphics: &Graphics, assets: &Assets) -> Shader;

    // Get the required mesh attributes that we need to render a surface
    // If a surface does not support these attributes, it will not be rendered
    fn attributes() -> MeshAttributes {
        MeshAttributes::all()
    }

    // Get the depth config for this material
    fn depth_config() -> Option<DepthConfig> {
        Some(DepthConfig {
            compare: CompareFunction::Less,
            write_enabled: true,
            depth_bias_constant: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        })
    }

    // Get the stencil testing for this material
    fn stencil_config() -> Option<StencilConfig> {
        None
    }

    // Get the rasterizer config for this materil
    fn primitive_config() -> PrimitiveConfig {
        PrimitiveConfig::Triangles {
            winding_order: WindingOrder::Cw,
            cull_face: Some(graphics::Face::Front),
            wireframe: false,
        }
    }

    // Get the blend config for this material
    fn blend_config() -> Option<BlendConfig<SceneColor>> {
        None
    }

    // Does this material support casting shadows onto other surfaces?
    fn casts_shadows() -> CastShadowsMode<Self> {
        CastShadowsMode::Enabled(None)
    }

    // Should surfaces using this material use frustum culling?
    fn frustum_culling() -> bool {
        true
    }

    // Fetch the required resources from the world
    fn fetch<'w>(world: &'w World) -> Self::Resources<'w>;

    // Set the static bindings
    fn set_global_bindings<'r, 'w>(
        _resources: &'r mut Self::Resources<'w>,
        _group: &mut BindGroup<'r>,
        _default: &DefaultMaterialResources<'r>,
    ) {
    }

    // Set the per instance bindings
    fn set_instance_bindings<'r, 'w>(
        &self,
        _resources: &'r mut Self::Resources<'w>,
        _default: &DefaultMaterialResources<'r>,
        _group: &mut BindGroup<'r>,
    ) {
    }

    // Set the per surface bindings
    fn set_surface_bindings<'r, 'w>(
        _renderer: &Renderer,
        _resources: &'r mut Self::Resources<'w>,
        _default: &mut DefaultMaterialResources<'w>,
        _query: &Self::Query<'w>,
        _group: &mut BindGroup<'r>,
    ) {
    }

    // Set the required push constants
    fn set_push_constants<'r, 'w>(
        &self,
        _renderer: &Renderer,
        _resources: &'r mut Self::Resources<'w>,
        _default: &DefaultMaterialResources<'r>,
        _query: &Self::Query<'w>,
        _push_constants: &mut PushConstants<ActiveScenePipeline>,
    ) {
    }
}
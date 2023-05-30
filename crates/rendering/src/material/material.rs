use crate::{
    ActiveScenePipeline, DefaultMaterialResources, MeshAttributes, RenderPath, Renderer, SceneColorLayout, Direct,
};
use assets::Assets;


use graphics::{
    BindGroup, BlendConfig, CompareFunction, DepthConfig, Graphics, PrimitiveConfig, PushConstants,
    Shader, StencilConfig, WindingOrder,
};

use world::World;

// A material is what defines the physical properties of surfaces whenever we draw them onto the screen
// Materials correspond to a specific WGPU render pipeline based on it's config parameters
pub trait Material: 'static + Sized + Sync + Send {
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

    // Get the rasterizer config for this materil
    fn primitive_config() -> PrimitiveConfig {
        PrimitiveConfig::Triangles {
            winding_order: WindingOrder::Cw,
            cull_face: Some(graphics::Face::Front),
            wireframe: false,
        }
    }

    // Does this material support casting shadows onto other surfaces?
    fn casts_shadows() -> bool {
        true
    }

    // Should surfaces using this material use frustum culling?
    fn frustum_culling() -> bool {
        true
    }

    // Fetch the required resources from the world
    fn fetch(world: &World) -> Self::Resources<'_>;

    // Set the static bindings
    fn set_global_bindings<'r>(
        _resources: &'r mut Self::Resources<'_>,
        _group: &mut BindGroup<'r>,
        _default: &DefaultMaterialResources<'r>,
    ) {
    }

    // Set the per instance bindings
    fn set_instance_bindings<'r>(
        &self,
        _resources: &'r mut Self::Resources<'_>,
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
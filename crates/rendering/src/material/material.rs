use crate::{
    DefaultMaterialResources, MeshAttributes, RenderPath, Renderer, SceneColorLayout, Direct, Pass, SubSurface,
};
use assets::Assets;


use graphics::{
    BindGroup, BlendConfig, CompareFunction, DepthConfig, Graphics, PrimitiveConfig, PushConstants,
    Shader, StencilConfig, WindingOrder, ActiveRenderPipeline,
};

use world::World;

// A material is what defines the physical properties of surfaces whenever we draw them onto the screen
// Materials correspond to a specific WGPU render pipeline based on it's config parameters
pub trait Material: 'static + Sized + Sync + Send {
    type Resources<'w>: 'w;
    type RenderPath: RenderPath;
    type Settings<'s>;
    type Query<'a>: ecs::QueryLayoutRef;

    // Checks if a material can be rendered with the given render path
    // If not, this will return a warning when trying to render a mesh
    fn is_render_path_supported<RP: RenderPath>(settings: &Self::Settings<'_>) -> bool { false }

    // Checks if a material can be used for rendering for a specific pass
    // If not, this will not call the following "shader()" method to setup shader
    fn is_pass_supported<P: Pass>(settings: &Self::Settings<'_>) -> bool { false }

    // Create a shader for this material for a specific pass
    // You can return "None" to disable rendering for that specific pass
    fn shader<P: Pass>(settings: &Self::Settings<'_>, graphics: &Graphics, assets: &Assets) -> Option<Shader>;

    // Get the required mesh attributes that we need to render a surface
    // If a surface does not support these attributes, it will not be rendered
    fn attributes<P: Pass>() -> MeshAttributes {
        MeshAttributes::all()
    }

    // Get the rasterizer config for this materil
    fn primitive_config<P: Pass>() -> PrimitiveConfig {
        PrimitiveConfig::Triangles {
            winding_order: WindingOrder::Cw,
            cull_face: Some(graphics::Face::Front),
            wireframe: false,
        }
    }

    // Should surfaces using this material use culling?
    fn cull<P: Pass>() -> bool {
        true
    }

    // Fetch the required resources from the world
    fn fetch<'w, P: Pass>(world: &'w World) -> Self::Resources<'w>;

    // Set the static bindings
    fn set_global_bindings<'r, P: Pass>(
        _resources: &'r mut Self::Resources<'_>,
        _group: &mut BindGroup<'r>,
        _default: &DefaultMaterialResources<'r>,
    ) {
    }

    // Set the per instance bindings
    fn set_instance_bindings<'r, P: Pass>(
        &self,
        _resources: &'r mut Self::Resources<'_>,
        _default: &DefaultMaterialResources<'r>,
        _group: &mut BindGroup<'r>,
    ) {
    }

    // Set the per surface bindings
    fn set_surface_bindings<'r, 'w, P: Pass>(
        _renderer: &Renderer,
        _resources: &'r mut Self::Resources<'w>,
        _default: &DefaultMaterialResources<'w>,
        _query: &Self::Query<'w>,
        _group: &mut BindGroup<'r>,
    ) {
    }

    // Set the required push constants
    fn set_push_constants<'r, 'w, P: Pass>(
        &self,
        _renderer: &Renderer,
        _resources: &'r mut Self::Resources<'w>,
        _default: &DefaultMaterialResources<'r>,
        _query: &Self::Query<'w>,
        _push_constants: &mut PushConstants<ActiveRenderPipeline<P::C, P::DS>>,
    ) {
    }
}
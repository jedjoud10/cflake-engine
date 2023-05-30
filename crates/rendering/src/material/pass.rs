use assets::Assets;
use graphics::{ColorLayout, DepthStencilLayout, RenderPass, BindGroup, Graphics, Shader, PrimitiveConfig, WindingOrder, PushConstants, ActiveRenderPipeline, ActivePipeline};
use world::World;
use crate::{RenderPath, DefaultMaterialResources, SceneColorLayout, SceneDepth, MeshAttributes, Renderer};

// Render pass that will render the scene
pub struct SceneRenderPass;

// Render pass that will render the shadows of the scene from the light POV
pub struct SceneShadowPass;

// Generalized render pass from within the rendering system
// This will be implemented for the RenderPass and ShadowPass structs
trait Pass {
    // Render-pass color and depth/stencil layouts
    type ColorLayout: ColorLayout;
    type DepthStencilLayout: DepthStencilLayout;

    type RenderPipeline;
    type ActiveRenderPipeline: ActivePipeline;
}

// A render pipeline is what the user will want to implement for to customize rendering
trait Pipeline<P: Pass> {
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
        _push_constants: &mut PushConstants<P::ActiveRenderPipeline>,
    ) {
    }
}
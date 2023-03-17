use crate::{
    AlbedoMap, CameraBuffer, EnabledMeshAttributes, NormalMap,
    Renderer, SceneBuffer, SceneColor, TimingBuffer, MaskMap,
};
use assets::Assets;
use graphics::{
    BindGroup, BlendConfig, CompareFunction, DepthConfig, Graphics,
    PrimitiveConfig, PushConstants, Shader, StencilConfig,
    WindingOrder,
};

use world::World;

// These are the default settings that we pass to each material
pub struct DefaultMaterialResources<'a> {
    // Main scene uniform buffers
    // TODO: Make use of crevice to implement Std130, Std140
    pub camera_buffer: &'a CameraBuffer,
    pub timing_buffer: &'a TimingBuffer,
    pub scene_buffer: &'a SceneBuffer,

    // Main scene textures
    pub white: &'a AlbedoMap,
    pub black: &'a AlbedoMap,
    pub normal: &'a NormalMap,
    pub mask: &'a MaskMap,

    // Currently used indicies
    pub material_index: usize,
    pub draw_call_index: usize,
}

// A material is what defines the physical properties of surfaces whenever we draw them onto the screen
// Materials correspond to a specific WGPU render pipeline based on it's config parameters
pub trait Material: 'static + Sized {
    // The resources that we need to fetch from the world to set the descriptor sets
    type Resources<'w>: 'w;

    // Create a shader for this material
    fn shader(graphics: &Graphics, assets: &mut Assets) -> Shader;

    // Get the required mesh attributes that we need to render a surface
    // If a surface does not support these attributes, it will not be rendered
    fn attributes() -> EnabledMeshAttributes {
        EnabledMeshAttributes::all()
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
    fn casts_shadows() -> bool {
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
        _group: &mut BindGroup<'r>,
    ) {
    }

    // Set the required push constants
    fn set_push_constants<'r, 'w>(
        &self,
        _renderer: &Renderer,
        _resources: &'r mut Self::Resources<'w>,
        _default: &DefaultMaterialResources<'r>,
        _push_constants: &mut PushConstants,
    ) {
    }
}

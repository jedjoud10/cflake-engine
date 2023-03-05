use crate::{
    AlbedoMap, CameraBuffer, CameraUniform, EnabledMeshAttributes,
    Mesh, NormalMap, Renderer, SceneBuffer, SceneColor, SceneUniform,
    TimingBuffer, TimingUniform,
};
use assets::Assets;
use graphics::{
    BindGroup, BlendConfig, CompareFunction, Compiled, DepthConfig,
    FragmentModule, Graphics, Normalized, PrimitiveConfig,
    PushConstants, StencilConfig, Texture2D, UniformBuffer,
    VertexModule, WindingOrder, RGBA,
};
use world::World;

// These are the default resources that we pass to any/each material
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

    // Default sky gradient texture
    pub sky_gradient: &'a AlbedoMap,
}

// A material is what defines the physical properties of surfaces whenever we draw them onto the screen
// Materials correspond to a specific WGPU render pipeline based on it's config parameters
pub trait Material: 'static + Sized {
    // The resources that we need to fetch from the world to set the descriptor sets
    type Resources<'w>: 'w;

    // Load the vertex module and process it
    fn vertex(
        graphics: &Graphics,
        assets: &mut Assets,
    ) -> Compiled<VertexModule>;

    // Load the fragment module and process it
    fn fragment(
        graphics: &Graphics,
        assets: &mut Assets,
    ) -> Compiled<FragmentModule>;

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

    // Fetch the required resources from the world
    fn fetch<'w>(world: &'w World) -> Self::Resources<'w>;

    // Set the static bindings
    fn set_global_bindings<'r, 'w>(
        resources: &'r mut Self::Resources<'w>,
        default: &DefaultMaterialResources<'r>,
        group: &mut BindGroup<'r>,
    ) {
    }

    // Set the per instance bindings
    fn set_instance_bindings<'r, 'w>(
        &self,
        resources: &'r mut Self::Resources<'w>,
        default: &DefaultMaterialResources<'r>,
        group: &mut BindGroup<'r>,
    ) {
    }

    // Set the per surface bindings
    fn set_surface_bindings<'r, 'w>(
        renderer: &Renderer,
        resources: &'r mut Self::Resources<'w>,
        default: &DefaultMaterialResources<'r>,
        group: &mut BindGroup<'r>,
    ) {
    }

    // Set push constants (per surface)
    fn set_push_constants<'r, 'w>(
        &self,
        renderer: &Renderer,
        resources: &'r mut Self::Resources<'w>,
        default: &DefaultMaterialResources<'r>,
        push_constants: &mut PushConstants,
    ) {
    }
}

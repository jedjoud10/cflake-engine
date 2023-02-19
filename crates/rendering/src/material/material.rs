use crate::{
    AlbedoMap, CameraBuffer, CameraUniform, EnabledMeshAttributes,
    Mesh, NormalMap, Renderer, SceneBuffer, SceneUniform,
    TimingBuffer, TimingUniform,
};
use assets::Assets;
use graphics::{
    BindGroup, BlendConfig, Compiled, DepthConfig, FragmentModule,
    FrontFace, Graphics, Normalized, PrimitiveConfig, StencilConfig,
    Texture2D, UniformBuffer, VertexModule, RGBA,
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
}

// A material is what defines the physical properties of surfaces whenever we draw them onto the screen
// Materials correspond to a specific WGPU render pipeline based on it's config parameters
pub trait Material: 'static + Sized {
    // The resources that we need to fetch from the world to set the descriptor sets
    type Resources<'w>: 'w;

    // Load the vertex module and process it
    fn vertex(
        graphics: &Graphics,
        assets: &Assets,
    ) -> Compiled<VertexModule>;

    // Load the fragment module and process it
    fn fragment(
        graphics: &Graphics,
        assets: &Assets,
    ) -> Compiled<FragmentModule>;

    // Get the required mesh attributes that we need to render a surface
    // If a surface does not support these attributes, it will not be rendered
    fn attributes() -> EnabledMeshAttributes {
        EnabledMeshAttributes::all()
    }

    // Get the depth config for this material
    fn depth_config() -> Option<DepthConfig> {
        None
    }

    // Get the stencil testing for this material
    fn stencil_config() -> Option<StencilConfig> {
        None
    }

    // Get the rasterizer config for this materil
    fn primitive_config() -> PrimitiveConfig {
        PrimitiveConfig::Triangles {
            winding_order: FrontFace::Ccw,
            cull_face: None,
            wireframe: false,
        }
    }

    // Get the blend config for this material
    fn blend_config() -> Option<BlendConfig> {
        None
    }

    // Fetch the required resources from the world
    fn fetch<'w>(world: &'w World) -> Self::Resources<'w>;

    // Set the static bindings
    fn set_global_bindings<'w>(
        resources: &mut Self::Resources<'w>,
        default: &DefaultMaterialResources<'w>,
        group: &mut BindGroup<'w>,
    ) {
    }

    // Set the per instance bindings
    fn set_instance_bindings<'w>(
        &self,
        resources: &mut Self::Resources<'w>,
        default: &DefaultMaterialResources<'w>,
        group: &BindGroup<'w>,
    ) {
    }

    // Set the per surface bindings
    fn set_surface_bindings<'w>(
        renderer: &Renderer,
        resources: &mut Self::Resources<'w>,
        default: &DefaultMaterialResources<'w>,
        group: &BindGroup<'w>,
    ) {
    }
}

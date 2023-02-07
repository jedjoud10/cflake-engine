use assets::Assets;
use graphics::{
    BlendConfig, Compiled, DepthConfig, FragmentModule, Graphics, PrimitiveConfig,
    StencilConfig, VertexModule, UniformBuffer, BindingConfig,
};
use world::World;
use crate::{EnabledMeshAttributes, Mesh, Renderer, CameraUniform, TimingUniform, SceneUniform};

// These are the default resources that we pass to any/each material
pub struct DefaultMaterialResources<'a> { 
    pub camera_buffer: &'a UniformBuffer<CameraUniform>,
    pub timing_buffer: &'a UniformBuffer<TimingUniform>,
    pub scene_buffer: &'a UniformBuffer<SceneUniform>,
}

// A material is what defines the physical properties of surfaces whenever we draw them onto the screen
// Materials correspond to a specific Vulkan pipeline based on it's config parameters
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
    fn attributes() -> EnabledMeshAttributes;

    // Get the depth config for this material
    fn depth_config() -> DepthConfig {
        DepthConfig {
            depth_write_enable: false,
            depth_clamp_enable: false,
            depth_test: None,
            depth_bias: None,
            depth_bounds: None,
        }
    }

    // Get the stencil testing for this material
    fn stencil_config() -> Option<StencilConfig> {
        None
    }

    // Get the rasterizer config for this materil
    fn primitive_config() -> PrimitiveConfig {
    }

    // Get the blend config for this material
    fn blend_config() -> Option<BlendConfig> {
        None
    }

    // Fetch the property block resources
    fn fetch<'w>(world: &'w World) -> Self::Resources<'w>;

    // Set the global bindings and uniforms required
    fn set_global_bindings<'w: 'ds, 'ds>(
        resources: &mut Self::Resources<'w>,
        default: &DefaultMaterialResources,
    ) {
        // check if camera desc set is present in pipeline
            // set it if it is
        // check if scene des set is present in pipeline
            // set it if it is
        
    }

    // Sets the bindings related to surface only
    fn set_surface_bindings<'w: 'ds, 'ds>(
        renderer: Renderer,
        resources: &mut Self::Resources<'w>,
        default: &DefaultMaterialResources,
    ) {}

    // This will only be called whenever we switch instances
    fn set_instance_bindings<'w: 'ds, 'ds>(
        &self,
        resources: &mut Self::Resources<'w>,
        default: &DefaultMaterialResources,
    ) {}
}
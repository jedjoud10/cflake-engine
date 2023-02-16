use assets::Assets;
use graphics::{
    BlendConfig, Compiled, DepthConfig, FragmentModule, Graphics, PrimitiveConfig,
    StencilConfig, VertexModule, UniformBuffer, BindingConfig, FrontFace,
};
use world::World;
use crate::{EnabledMeshAttributes, Mesh, Renderer, CameraUniform, TimingUniform, SceneUniform};

// These are the default resources that we pass to any/each material
pub struct DefaultMaterialResources<'a> { 
    // Main scene uniform buffers
    pub camera_buffer: &'a UniformBuffer<CameraUniform>,
    pub timing_buffer: &'a UniformBuffer<TimingUniform>,
    pub scene_buffer: &'a UniformBuffer<SceneUniform>,

    // Main scene textures
}

// A material is what defines the physical properties of surfaces whenever we draw them onto the screen
// Materials correspond to a specific Vulkan pipeline based on it's config parameters
pub trait Material: 'static + Sized {
    // The resources that we need to fetch from the world to set the descriptor sets
    type Resources<'w>: 'w;

    // Glboal, surface, and mesh descriptor sets
    type GlobalGroup<'a>;
    type InstanceGroup<'a>;
    type SurfaceGroup<'a>;

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
            wireframe: false
        }
    }

    // Get the blend config for this material
    fn blend_config() -> Option<BlendConfig> {
        None
    }

    // Get the global bind group required
    fn get_global_bind_group<'w>(
        resources: &mut Self::Resources<'w>,
        default: &DefaultMaterialResources,
    ) -> Self::GlobalGroup<'w> {
        todo!()
    }

    // Get the instance bind group
    fn get_instance_bind_group<'w>(
        &self,
        resources: &mut Self::Resources<'w>,
        default: &DefaultMaterialResources,
    ) -> Self::InstanceGroup<'w> {
        todo!()
    }

    // Get the surface bind group
    fn get_surface_bindings<'w>(
        renderer: Renderer,
        resources: &mut Self::Resources<'w>,
        default: &DefaultMaterialResources,
    ) -> Self::SurfaceGroup<'w> {
        todo!()
    }
}
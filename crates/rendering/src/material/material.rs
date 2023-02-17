use assets::Assets;
use graphics::{
    BlendConfig, Compiled, DepthConfig, FragmentModule, Graphics, PrimitiveConfig,
    StencilConfig, VertexModule, UniformBuffer, BindingConfig, FrontFace,
};
use world::World;
use crate::{EnabledMeshAttributes, Mesh, Renderer, CameraUniform, TimingUniform, SceneUniform, CameraBuffer, TimingBuffer, SceneBuffer};

// These are the default resources that we pass to any/each material
pub struct DefaultMaterialResources<'a> { 
    // Main scene uniform buffers
    pub camera_buffer: &'a CameraBuffer,
    pub timing_buffer: &'a TimingBuffer,
    pub scene_buffer: &'a SceneBuffer,

    // Main scene textures
}

// A binder is used to bind multiple values to the current render pass
// Bindes must have a specific WGPU layout that is used to create the graphics pipeline
pub trait Binder: 'static + Sized {
    // Nice interface structs that will contains the lifetimed data 
    type Instance<'a>;
    type Global<'a>;
    type Surface<'a>;

    // Statically typed pipeline layout
    // This contains the bind group entries and push constant ranges
    fn layout() -> ();
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
    ) {
        todo!()
    }

    // Get the instance bind group
    fn get_instance_bind_group<'w>(
        &self,
        resources: &mut Self::Resources<'w>,
        default: &DefaultMaterialResources,
    ) {
        todo!()
    }

    // Get the surface bind group
    fn get_surface_bindings<'w>(
        renderer: Renderer,
        resources: &mut Self::Resources<'w>,
        default: &DefaultMaterialResources,
    ) {
        todo!()
    }
}
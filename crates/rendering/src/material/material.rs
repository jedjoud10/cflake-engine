use assets::Assets;
use graphics::{
    BlendConfig, CompareOp, Compiled, DepthConfig, DescriptorSet,
    FaceCullMode, FragmentModule, Graphics, Primitive, Processed,
    StencilConfig, VertexModule,
};
use world::World;

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
    fn required_mesh_attributes() -> ();

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
    fn stencil_config() -> StencilConfig {
        StencilConfig(None)
    }

    // Get the rasterizer config for this materil
    fn primitive_mode() -> Primitive {
        Primitive::Triangles {
            cull: None,
            wireframe: false,
        }
    }

    // Get the blend config for this material
    fn blend_config() -> BlendConfig {
        BlendConfig {
            logic_operation: None,
            attachments: None,
        }
    }

    // Fetch the property block resources
    fn fetch<'w>(world: &'w World) -> Self::Resources<'w>;

    // Set the global and static instance descriptor sets
    fn get_static_descriptor_set<'w: 'ds, 'ds>(
        resources: &mut Self::Resources<'w>,
    ) {}

    // Set the uniforms for this property block right before we render our surface
    fn get_surface_descriptor_set<'w: 'ds, 'ds>(
        resources: &mut Self::Resources<'w>,
    ) {}

    // With the help of the fetched resources, set the uniform properties for a unique material instance
    // This will only be called whenever we switch instances
    fn get_instance_descriptor_set<'w: 'ds, 'ds>(
        resources: &mut Self::Resources<'w>,
        instance: &Self,
    ) {}
}
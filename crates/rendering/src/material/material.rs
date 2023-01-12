use assets::Assets;
use graphics::{
    BlendConfig, CompareOp, Compiled, DepthConfig, DescriptorSet,
    FaceCullMode, FragmentModule, Graphics, Primitive,
    StencilConfig, VertexModule,
};
use world::World;

use crate::{EnabledMeshAttributes, Mesh, Renderer};

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
    fn required_mesh_attributes() -> EnabledMeshAttributes;

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
    fn primitive() -> Primitive {
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

    // Get the global / static descriptor
    fn get_static_descriptor_set<'w: 'ds, 'ds>(
        resources: &mut Self::Resources<'w>,
    ) {}

    // Get the descriptor for per-mesh rendering
    fn get_surface_descriptor_set<'w: 'ds, 'ds>(
        renderer: Renderer,
        resources: &mut Self::Resources<'w>,
    ) {}

    // This will only be called whenever we switch instances
    fn get_instance_descriptor_set<'w: 'ds, 'ds>(
        resources: &mut Self::Resources<'w>,
        instance: &Self,
    ) {}
}
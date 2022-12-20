use assets::Assets;
use graphics::{VertexModule, FragmentModule, DepthConfig, CompareOp, StencilConfig, Primitive, FaceCullMode, BlendConfig, DescriptorSet};

// A material is what defines the physical properties of surfaces whenever we draw them onto the screen
// Materials correspond to a specific Vulkan pipeline based on it's config parameters
pub trait Material: 'static + Sized {
    // The resources that we need to fetch from the world to set the descriptor sets
    type Resources<'w>: 'w;

    // Static scene descriptor set
    type SceneDescriptorSet<'w>: 'w + DescriptorSet; 
    
    // Instance descriptor set
    type InstanceDescriptorSet<'w>: 'w + DescriptorSet; 
    
    // Surface descriptor set
    type SurfaceDescriptorSet<'w>: 'w + DescriptorSet; 

    // Load the vertex module
    fn vertex_module(assets: &Assets) -> VertexModule;

    // Load the fragment module
    fn fragment_module(assets: &Assets) -> FragmentModule;

    // Get the required mesh attributes that we need to render a surface
    fn required_mesh_attributes() -> ();

    // Get the depth config for this material
    fn depth_config() -> DepthConfig {
        DepthConfig { 
            depth_write_enable: true,
            depth_clamp_enable: false,
            depth_test: Some(CompareOp::Less),
            depth_bias: None,
            depth_bounds: None
        }
    }
    
    // Get the stencil testing for this material
    fn stencil_config() -> StencilConfig {
        StencilConfig(None)
    }

    // Get the rasterizer config for this materil
    fn primitive_mode() -> Primitive {
        Primitive::Triangles {
            cull: Some(FaceCullMode::Back(true)),
            wireframe: false,
        }
    }

    // Get the blend config for this material
    fn blend_config() -> BlendConfig {
        todo!()
    }

    // Set the global and static instance descriptor sets
    fn get_static_descriptor_set<'w>(
        resources: &mut Self::Resources<'w>,
    ) -> Self::SceneDescriptorSet<'w>;

    // Set the uniforms for this property block right before we render our surface
    fn get_surface_descriptor_set<'w>(
        resources: &mut Self::Resources<'w>,
    ) -> Self::SurfaceDescriptorSet<'w>;

    // With the help of the fetched resources, set the uniform properties for a unique material instance
    // This will only be called whenever we switch instances
    fn get_instance_descriptor_set<'w>(
        resources: &mut Self::Resources<'w>,
        instance: &Self,
    ) -> Self::InstanceDescriptorSet<'w>;
}
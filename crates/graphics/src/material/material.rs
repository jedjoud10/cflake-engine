use world::World;
use crate::{DepthConfig, BlendConfig, PrimitiveMode, CompareOp, FaceCullMode, StencilTest};


// A material is what defines the physical properties of surfaces whenever we draw them onto the screen
// Materials correspond to a specific Vulkan pipeline based on it's config parameters
pub trait Material: 'static + Sized {
    // The resources that we need to fetch from the world to set the descriptor sets
    type Resources<'w>: 'w;

    // Static scene descriptor set
    type SceneDescSet; 
    
    // Instance descriptor set
    type InstanceDescSet; 
    
    // Surface descriptor set
    type SurfaceDescSet; 

    // Get the required mesh attributes that we need to render a surface
    fn required_mesh_attributes() -> ();

    // Get the depth config for this material
    fn depth_config(&self) -> DepthConfig {
        DepthConfig { 
            depth_write_enable: true,
            depth_clamp_enable: false,
            depth_test: Some(CompareOp::Less),
            depth_bias: None,
            depth_bounds: None
        }
    }
    
    // Get the stencil testing for this material
    fn stencil_testing(&self) -> Option<StencilTest> {
        None
    }

    // Get the rasterizer config for this materil
    fn primitive_mode(&self) -> PrimitiveMode {
        PrimitiveMode::Triangles {
            cull: Some(FaceCullMode::Back(true)),
        }
    }

    // Get the blend config for this material
    fn blend_config(&self) -> BlendConfig {
        todo!()
    }

    // Set the global and static instance descriptor sets
    fn set_static_properties<'a>(
        resources: &mut Self::Resources<'a>,
    );

    // Set the uniforms for this property block right before we render our surface
    fn set_surface_properties<'a>(
        resources: &mut Self::Resources<'a>,
    );

    // With the help of the fetched resources, set the uniform properties for a unique material instance
    // This will only be called whenever we switch instances
    fn set_instance_properties<'a>(
        resources: &mut Self::Resources<'a>,
        instance: &Self,
    );
}
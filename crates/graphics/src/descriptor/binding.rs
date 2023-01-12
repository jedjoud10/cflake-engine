use vulkan::vk;
use crate::UntypedTexel;

// This is the bindinglayout for a specific resource that we will pass along later
pub enum BindingType {
    // A UBO that will be passed to the shader
    UBO {
        binding: u32,
    },

    // A combined image sampler that will be passed to the shader
    Sampler {
        binding: u32,
        untyped: UntypedTexel,
    },
}

// This is the specific shader modules that have access to the binding
pub enum ShaderModuleView {
    Vertex, Fragment, All,
}

// A single binding layout for a resource
pub struct BindingLayout {
    pub binding_type: BindingType,
    pub view: ShaderModuleView,
}

// This trait is implemented for resources that can be viewed within shaders
pub trait BindableResource {
    
}


use graphics_derive::Bindings;
use vulkan::vk;
use crate::{Sampler, Normalized, RGBA, Texture2D, UntypedTexel};


pub enum BindingType {
    UBO {
        binding: u32,
        element: u32,
    },
    Sampler {
        binding: u32,
        untyped: UntypedTexel,
    },
}

#[repr(u32)]
pub enum BindingFrequency {
    Global, Instance, Mesh
}

pub enum ShaderModuleView {
    Vertex, Fragment, All,
}

// A single binding layout for a resource
pub struct BindingLayout {
    pub binding_type: BindingType,
    pub frequency: BindingFrequency,
    pub view: ShaderModuleView,
}

// The predefined layout of a descriptor set
pub trait DescriptorSetLayout {
    fn descriptors() -> Vec<BindingLayout>;
}

// A descriptor set that we can bind to a pipeline
pub trait DescriptorSet<'a> {}


#[derive(Bindings)]
pub struct Test<'a> {
	#[sampler(binding = 0)]		
	#[frequency(mesh)]
    #[view(fragment)]
    diffuse_map: Sampler<'a, Texture2D<RGBA<Normalized<u8>>>>,

    #[sampler(binding = 1)]		
	#[frequency(mesh)]
    #[view(fragment, vertex)]
    global_map: Sampler<'a, Texture2D<RGBA<Normalized<u8>>>>,

    /*
    #[push_constants]
    #[frequency(mesh)]
    #[view(fragment, vertex)]
    data: f32,
    */
}
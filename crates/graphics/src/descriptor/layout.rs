use vulkan::vk;

// The predefined layout of a descriptor set
pub unsafe trait DescriptorLayout {}

unsafe impl<T> DescriptorLayout for T {}

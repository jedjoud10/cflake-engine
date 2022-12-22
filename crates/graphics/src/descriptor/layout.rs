use vulkan::vk;

// The predefined layout of a descriptor set
// Automatically implemented whenever we implement DescriptorSet derive
pub unsafe trait DescriptorLayout {
    fn layout() -> vk::DescriptorSetLayout;
}
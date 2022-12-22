use crate::DescriptorLayout;

// A descriptor set that we can bind to a pipeline
// This trait will automatically be implemented with the derive Descriptor macro
pub unsafe trait DescriptorSet: DescriptorLayout {}
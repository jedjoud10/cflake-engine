use crate::DescriptorLayout;

// A descriptor set that we can bind to a pipeline
pub unsafe trait DescriptorSet<'a>: DescriptorLayout {}

unsafe impl<'a, T> DescriptorSet<'a> for T {}
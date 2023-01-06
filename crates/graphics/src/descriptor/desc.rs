// A single descriptor for a resource
pub unsafe trait Descriptor<'a> {}

// The predefined layout of a descriptor set
pub unsafe trait DescriptorLayout {}

// A descriptor set that we can bind to a pipeline
pub unsafe trait DescriptorSet<'a>: DescriptorLayout {}
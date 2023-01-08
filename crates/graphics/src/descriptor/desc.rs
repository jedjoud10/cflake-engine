// A single descriptor for a resource
pub trait Descriptor<'a> {}

// The predefined layout of a descriptor set
pub trait DescriptorLayout {}

// A descriptor set that we can bind to a pipeline
pub trait DescriptorSet<'a> {}
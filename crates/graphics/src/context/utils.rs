use bytemuck::{Pod, Zeroable};

// Plain old data type internally used by buffers and other types
pub trait Content:
    Zeroable + Pod + Clone + Copy + Sync + Send + 'static
{
}
impl<T: Clone + Copy + Sync + Send + Zeroable + Pod + 'static> Content
    for T
{
}

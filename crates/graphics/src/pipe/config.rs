use vulkan::vk;

// Comparison operator that represents the raw Vulkan comparison modes
// Equivalent to vk::CompareOp
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Never = 0,
    Less,
    Equal,
    LessThanOrEquals,
    Greater,
    NotEqual,
    GreaterThanOrEquals,
    Always,
}
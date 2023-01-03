use vulkan::vk;

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

// Equivalent to vk::LogicOp
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LogicOp {
    Clear = 0,
    And,
    AndReverse,
    Copy,
    AndInverted,
    NoOp,
    Xor,
    Or,
    Nor,
    Equivalent,
    Invert,
    OrReverse,
    CopyInverted,
    OrInverted,
    Nand,
    Set,
}
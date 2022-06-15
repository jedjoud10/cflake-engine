#[repr(u32)]
#[derive(Clone)]
// Comparison type that represents the raw OpenGL comparison modes
pub enum Comparison {
    Always = gl::ALWAYS,
    Never = gl::NEVER,
    Less = gl::LESS,
    Equal = gl::EQUAL,
    LessThanOrEquals = gl::LEQUAL,
    Greater = gl::GREATER,
    NotEqual = gl::NOTEQUAL,
    GreaterThanOrEquals = gl::GEQUAL,
}
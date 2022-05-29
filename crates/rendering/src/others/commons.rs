// Common OpenGL wrapper types
#[repr(u32)]
#[derive(Clone)]
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

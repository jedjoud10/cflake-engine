// Blend mode factor source.
#[repr(u32)]
#[derive(Clone, Copy)]
pub enum Factor {
    Zero = gl::ZERO,
    One = gl::ONE,
    SrcColor = gl::SRC_COLOR,
    DstColor = gl::DST_COLOR,
    SrcAlpha = gl::SRC_ALPHA,
    DstAlpha = gl::DST_ALPHA,
    OneMinusSrcColor = gl::ONE_MINUS_SRC_COLOR,
    OneMinusDstColor = gl::ONE_MINUS_DST_COLOR,
    OneMinusSrcAlpha = gl::ONE_MINUS_SRC_ALPHA,
    OneMinusDstAlpha = gl::ONE_MINUS_DST_ALPHA,
}

// Blending mode when utilising alpha blending moment
#[derive(Clone, Copy)]
pub struct BlendMode {
    pub(super) s_factor: Factor,
    pub(super) d_factor: Factor,
}

impl BlendMode {
    pub fn with(s_factor: Factor, d_factor: Factor) -> Self {
        Self { s_factor, d_factor }
    }
}

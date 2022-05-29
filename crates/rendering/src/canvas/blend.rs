// Blend mode factor source.
#[derive(Clone, Copy)]
pub enum FactorMode {
    Zero,
    One,
    SrcColor,
    DstColor,
    SrcAlpha,
    DstAlpha,
}

// The factor type that we will use for the sFactor and dFactor values
#[derive(Clone, Copy)]
pub struct Factor {
    mode: FactorMode,
    one_minus_value: bool,
}

impl Factor {
    fn with(mode: FactorMode, one_minus_value: bool) -> Self {
        Self { mode, one_minus_value }
    }

    // Convert the factor to the OpenGL raw factor
    pub(super) fn convert(&self) -> u32 {
        match (&self.mode, self.one_minus_value) {
            (FactorMode::Zero, true) => gl::ONE,
            (FactorMode::Zero, false) => gl::ZERO,
            (FactorMode::One, true) => gl::ZERO,
            (FactorMode::One, false) => gl::ONE,
            (FactorMode::SrcColor, true) => gl::ONE_MINUS_SRC_COLOR,
            (FactorMode::SrcColor, false) => gl::SRC_COLOR,
            (FactorMode::DstColor, true) => gl::ONE_MINUS_DST_COLOR,
            (FactorMode::DstColor, false) => gl::DST_COLOR,
            (FactorMode::SrcAlpha, true) => gl::ONE_MINUS_SRC_ALPHA,
            (FactorMode::SrcAlpha, false) => gl::SRC_ALPHA,
            (FactorMode::DstAlpha, true) => gl::ONE_MINUS_DST_ALPHA,
            (FactorMode::DstAlpha, false) => gl::DST_ALPHA,
        }
    }
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

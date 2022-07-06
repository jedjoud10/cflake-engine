// Texel filters that are applied to the texture's mininifcation and magnification parameters
#[derive(Clone, Copy)]
pub enum Filter {
    Nearest,
    Linear,
}

// Wrapping mode utilised by TEXTURE_WRAP_R and TEXTURE_WRAP_T
#[derive(Clone, Copy)]
pub enum Wrap {
    ClampToEdge,
    ClampToBorder(vek::Rgba<f32>),
    Repeat,
    MirroredRepeat,
}

// Some special sampling parameters for textures
#[derive(Clone, Copy)]
pub struct Sampling {
    pub filter: Filter,
    pub wrap: Wrap,
}

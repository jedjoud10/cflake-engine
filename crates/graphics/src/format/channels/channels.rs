// Untyped representation of texel channels
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TexelChannels {
    // Normal channel types at first
    One,
    Two,
    Four { swizzled: bool },

    // Either SRGBA, or SBGRA
    Srgba { swizzled: bool },

    // Always 1
    Depth,
    Stencil,
}

impl TexelChannels {
    // Count the number of channels that we have in total
    pub const fn count(&self) -> u32 {
        match self {
            Self::Depth | Self::Stencil | Self::One => 1,
            Self::Two => 2,
            Self::Srgba { .. } | Self::Four { .. } => 4,
        }
    }

    // Check if the R (X) and B (Z) channels are swizzled
    pub const fn is_swizzled(&self) -> bool {
        match self {
            TexelChannels::Four { swizzled }
            | TexelChannels::Srgba { swizzled } => *swizzled,
            _ => false
        }
    }
}

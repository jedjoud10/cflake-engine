use crate::{ChannelsType, ElementType, VectorChannels};
use paste::paste;
use wgpu::{TextureFormat, VertexFormat};

// Implement a function that will deserialize the element type for a specific channel type
// Only supported: R, Rg, RGBA
macro_rules! impl_texel_conversion_specific_channels {
    ($channel:ident, $function:ident) => {
        paste! {
            const fn [<handle_ $function _formats>](element: ElementType) -> Option<TextureFormat> {
                match element {
                    ElementType::Eight { signed, normalized } => Some(match (signed, normalized) {
                        (true, true) => TextureFormat::[<$channel 8Snorm>],
                        (true, false) => TextureFormat::[<$channel 8Sint>],
                        (false, true) => TextureFormat::[<$channel 8Unorm>],
                        (false, false) => TextureFormat::[<$channel 8Uint>],
                    }),
                    ElementType::Sixteen { signed, normalized } => Some(match (signed, normalized) {
                        (true, true) => TextureFormat::[<$channel 16Snorm>],
                        (true, false) => TextureFormat::[<$channel 16Sint>],
                        (false, true) => TextureFormat::[<$channel 16Unorm>],
                        (false, false) => TextureFormat::[<$channel 16Uint>],
                    }),
                    ElementType::ThirtyTwo { signed } => Some(match signed {
                        true => TextureFormat::[<$channel 32Sint>],
                        false => TextureFormat::[<$channel 32Uint>],
                    }),
                    ElementType::FloatSixteen => Some(TextureFormat::[<$channel 16Float>]),
                    ElementType::FloatThirtyTwo => Some(TextureFormat::[<$channel 32Float>]),
                    _ => None
                }
            }
        }
    };
}

// Converts the given vector channels to the proper texture format
pub const fn pick_texture_format_from_vector_channels(
    element: ElementType,
    channels: VectorChannels,
) -> Option<TextureFormat> {
    impl_texel_conversion_specific_channels!(R, r);
    impl_texel_conversion_specific_channels!(Rg, rg);
    impl_texel_conversion_specific_channels!(Rgba, rgba);

    // Handle BGRA formats by themselves since WGPU doesn't support all formats
    const fn handle_bgra_formats(
        element: ElementType,
    ) -> Option<TextureFormat> {
        match element {
            ElementType::Eight {
                signed: false,
                normalized: true,
            } => Some(TextureFormat::Bgra8Unorm),
            _ => None,
        }
    }

    match channels {
        VectorChannels::One => handle_r_formats(element),
        VectorChannels::Two => handle_rg_formats(element),
        VectorChannels::Four => handle_rgba_formats(element),
        VectorChannels::FourSwizzled => handle_bgra_formats(element),
        _ => None,
    }
}

// Converts the given depth channel to the proper format
pub const fn pick_texture_depth_format(
    element: ElementType,
) -> Option<TextureFormat> {
    match element {
        ElementType::Sixteen {
            signed: false,
            normalized: true,
        } => Some(TextureFormat::Depth16Unorm),
        ElementType::FloatThirtyTwo => {
            Some(TextureFormat::Depth32Float)
        }
        _ => None,
    }
}

// Converts the given stencil channel to the proper format
pub const fn pick_texture_stencil_format(
    element: ElementType,
) -> Option<TextureFormat> {
    match element {
        ElementType::Eight {
            signed: false,
            normalized: false,
        } => Some(TextureFormat::Stencil8),
        _ => None,
    }
}

// Converts the given data to the proper texel format
// This is called within the Texel::format function
// The given input might not always be supported (for example, RGB), and in which case, this function would return None
pub const fn pick_texture_format(
    element: ElementType,
    channels: ChannelsType,
) -> Option<TextureFormat> {
    match channels {
        ChannelsType::Vector(channels) => {
            pick_texture_format_from_vector_channels(
                element,
                channels,
            )
        }
        ChannelsType::Depth => {
            pick_texture_depth_format(element)
        }
        ChannelsType::Stencil => {
            pick_texture_stencil_format(element)
        }
    }
}

// Converts the given data to the proper vertex format 
// This is called within the Vertex::format()
// The given input might not always be supported (for example, XYZ), and in which case, this function would return None
pub const fn pick_vertex_format(
    element: ElementType,
    channels: VectorChannels,
) -> Option<VertexFormat> {
    match channels {
        VectorChannels::One => match element {
            ElementType::ThirtyTwo { signed } => Some(match signed {
                true => VertexFormat::Sint32,
                false => VertexFormat::Uint32,
            }),
            ElementType::FloatThirtyTwo => Some(VertexFormat::Float32),
            _ => None
        },
        VectorChannels::Two => match element {
            ElementType::Eight { signed, normalized } => Some(match (signed, normalized) {
                (true, true) => VertexFormat::Snorm8x2,
                (true, false) => VertexFormat::Sint8x2,
                (false, true) => VertexFormat::Unorm8x2,
                (false, false) => VertexFormat::Uint8x2,
            }),
            ElementType::Sixteen { signed, normalized } => Some(match (signed, normalized) {
                (true, true) => VertexFormat::Snorm16x2,
                (true, false) => VertexFormat::Sint16x2,
                (false, true) => VertexFormat::Unorm16x2,
                (false, false) => VertexFormat::Uint16x2,
            }),
            ElementType::ThirtyTwo { signed } => Some(match signed {
                true => VertexFormat::Sint32x2,
                false => VertexFormat::Uint32x2,
            }),
            ElementType::FloatSixteen => Some(VertexFormat::Float16x2),
            ElementType::FloatThirtyTwo => Some(VertexFormat::Float32x2),
            _ => None
        },
        VectorChannels::Three => match element {
            ElementType::ThirtyTwo { signed } => Some(match signed {
                true => VertexFormat::Sint32x3,
                false => VertexFormat::Uint32x3,
            }),
            ElementType::FloatThirtyTwo => Some(VertexFormat::Float32x3),
            _ => None
        },
        VectorChannels::Four => match element {
            ElementType::Eight { signed, normalized } => Some(match (signed, normalized) {
                (true, true) => VertexFormat::Snorm8x4,
                (true, false) => VertexFormat::Sint8x4,
                (false, true) => VertexFormat::Unorm8x4,
                (false, false) => VertexFormat::Uint8x4,
            }),
            ElementType::Sixteen { signed, normalized } => Some(match (signed, normalized) {
                (true, true) => VertexFormat::Snorm16x4,
                (true, false) => VertexFormat::Sint16x4,
                (false, true) => VertexFormat::Unorm16x4,
                (false, false) => VertexFormat::Uint16x4,
            }),
            ElementType::ThirtyTwo { signed } => Some(match signed {
                true => VertexFormat::Sint32x4,
                false => VertexFormat::Uint32x4,
            }),
            ElementType::FloatSixteen => Some(VertexFormat::Float16x4),
            ElementType::FloatThirtyTwo => Some(VertexFormat::Float32x4),
            _ => None
        },
        _ => None
    }
}
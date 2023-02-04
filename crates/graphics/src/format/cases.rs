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

// Converts the given vector channels to the proper vertex format
pub const fn pick_vertex_format_from_vector_channels(
    element: ElementType,
    channels: VectorChannels,
) -> Option<VertexFormat> {
    None
}

// Converts the given depth channel to the proper format
pub const fn pick_texture_depth_format(
    element_type: ElementType,
) -> Option<TextureFormat> {
    match element_type {
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
    element_type: ElementType,
) -> Option<TextureFormat> {
    match element_type {
        ElementType::Eight {
            signed: false,
            normalized: false,
        } => Some(TextureFormat::Stencil8),
        _ => None,
    }
}

// Converts the given data to the proper format
// This is called within the Texel::FORMAT and Vertex::FORMAT
// The given input might not always be supported (for example, RGB), and in which case, this function would return None
pub const fn pick_texture_format(
    element_type: ElementType,
    channels_type: ChannelsType,
) -> Option<TextureFormat> {
    match channels_type {
        ChannelsType::Vector(channels) => {
            pick_texture_format_from_vector_channels(
                element_type,
                channels,
            )
        }
        ChannelsType::Depth => {
            pick_texture_depth_format(element_type)
        }
        ChannelsType::Stencil => {
            pick_texture_stencil_format(element_type)
        }
    }
}

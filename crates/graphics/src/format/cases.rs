use crate::{ElementType, TexelChannels, VertexChannels, CompressionType};
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
pub const fn pick_texture_format_channels(
    element: ElementType,
    channels: TexelChannels,
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

    // Handle compressed formats early since I don't want to deal with them in the macro
    match element {
        ElementType::Compressed(compressed) => {
            match (channels, compressed) {
                (TexelChannels::One, CompressionType::BC4 { signed }) => {
                    return Some(match signed {
                        true => TextureFormat::Bc4RSnorm,
                        false => TextureFormat::Bc4RUnorm,
                    });
                },
                (TexelChannels::Two, CompressionType::BC5 { signed }) => {
                    return Some(match signed {
                        true => TextureFormat::Bc5RgSnorm,
                        false => TextureFormat::Bc5RgUnorm,
                    });
                },
                (TexelChannels::Four { swizzled: false }, CompressionType::UBC1) => {
                    return Some(TextureFormat::Bc1RgbaUnorm);
                },
                (TexelChannels::Four { swizzled: false }, CompressionType::UBC2) => {
                    return Some(TextureFormat::Bc2RgbaUnorm);
                },
                (TexelChannels::Four { swizzled: false }, CompressionType::UBC3) => {
                    return Some(TextureFormat::Bc3RgbaUnorm);
                },
                (TexelChannels::Four { swizzled: false }, CompressionType::UBC7) => {
                    return Some(TextureFormat::Bc7RgbaUnorm);
                },
                _ => {}
            }
        },
        _ => {}
    }

    match channels {
        TexelChannels::One => handle_r_formats(element),
        TexelChannels::Two => handle_rg_formats(element),
        TexelChannels::Four { swizzled } => match swizzled {
            true => handle_bgra_formats(element),
            false => handle_rgba_formats(element),
        },
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

pub const fn pick_texture_srgb_format(
    element: ElementType,
    swizzled: bool,
) -> Option<TextureFormat> {
    match element {
        ElementType::Eight { 
            signed: false,
            normalized: true
        } => Some(match swizzled {
            true => TextureFormat::Bgra8UnormSrgb,
            false => TextureFormat::Rgba8UnormSrgb,
        }),
        ElementType::Compressed(compression) => match compression {
            CompressionType::UBC1 => Some(TextureFormat::Bc1RgbaUnormSrgb),
            CompressionType::UBC2 => Some(TextureFormat::Bc2RgbaUnormSrgb),
            CompressionType::UBC3 => Some(TextureFormat::Bc3RgbaUnormSrgb),
            CompressionType::UBC7 => Some(TextureFormat::Bc7RgbaUnormSrgb),
            _ => None
        },
        _ => None
    }
}

// Converts the given data to the proper texel format
// This is called within the Texel::format function
// The given input might not always be supported (for example, RGB), and in which case, this function would return None
pub const fn pick_texture_format(
    element: ElementType,
    channels: TexelChannels,
) -> Option<TextureFormat> {
    match channels {
        TexelChannels::One
        | TexelChannels::Two
        | TexelChannels::Four { .. } => pick_texture_format_channels(element, channels),

        TexelChannels::Depth => pick_texture_depth_format(element),

        TexelChannels::Stencil => {
            pick_texture_stencil_format(element)
        }

        TexelChannels::Srgba { swizzled } => {
            pick_texture_srgb_format(element, swizzled)
        }
    }
}

// Converts the given data to the proper vertex format
// This is called within the Vertex::format()
// The given input might not always be supported (for example, XYZ), and in which case, this function would return None
pub const fn pick_vertex_format(
    element: ElementType,
    channels: VertexChannels,
) -> Option<VertexFormat> {
    match channels {
        VertexChannels::One => match element {
            ElementType::ThirtyTwo { signed } => Some(match signed {
                true => VertexFormat::Sint32,
                false => VertexFormat::Uint32,
            }),
            ElementType::FloatThirtyTwo => {
                Some(VertexFormat::Float32)
            }
            _ => None,
        },
        VertexChannels::Two => match element {
            ElementType::Eight { signed, normalized } => {
                Some(match (signed, normalized) {
                    (true, true) => VertexFormat::Snorm8x2,
                    (true, false) => VertexFormat::Sint8x2,
                    (false, true) => VertexFormat::Unorm8x2,
                    (false, false) => VertexFormat::Uint8x2,
                })
            }
            ElementType::Sixteen { signed, normalized } => {
                Some(match (signed, normalized) {
                    (true, true) => VertexFormat::Snorm16x2,
                    (true, false) => VertexFormat::Sint16x2,
                    (false, true) => VertexFormat::Unorm16x2,
                    (false, false) => VertexFormat::Uint16x2,
                })
            }
            ElementType::ThirtyTwo { signed } => Some(match signed {
                true => VertexFormat::Sint32x2,
                false => VertexFormat::Uint32x2,
            }),
            ElementType::FloatSixteen => {
                Some(VertexFormat::Float16x2)
            }
            ElementType::FloatThirtyTwo => {
                Some(VertexFormat::Float32x2)
            }
            _ => None,
        },
        VertexChannels::Three => match element {
            ElementType::ThirtyTwo { signed } => Some(match signed {
                true => VertexFormat::Sint32x3,
                false => VertexFormat::Uint32x3,
            }),
            ElementType::FloatThirtyTwo => {
                Some(VertexFormat::Float32x3)
            }
            _ => None,
        },
        VertexChannels::Four => match element {
            ElementType::Eight { signed, normalized } => {
                Some(match (signed, normalized) {
                    (true, true) => VertexFormat::Snorm8x4,
                    (true, false) => VertexFormat::Sint8x4,
                    (false, true) => VertexFormat::Unorm8x4,
                    (false, false) => VertexFormat::Uint8x4,
                })
            }
            ElementType::Sixteen { signed, normalized } => {
                Some(match (signed, normalized) {
                    (true, true) => VertexFormat::Snorm16x4,
                    (true, false) => VertexFormat::Sint16x4,
                    (false, true) => VertexFormat::Unorm16x4,
                    (false, false) => VertexFormat::Uint16x4,
                })
            }
            ElementType::ThirtyTwo { signed } => Some(match signed {
                true => VertexFormat::Sint32x4,
                false => VertexFormat::Uint32x4,
            }),
            ElementType::FloatSixteen => {
                Some(VertexFormat::Float16x4)
            }
            ElementType::FloatThirtyTwo => {
                Some(VertexFormat::Float32x4)
            }
            _ => None,
        },
        _ => None,
    }
}

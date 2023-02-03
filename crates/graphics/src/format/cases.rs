use crate::{ChannelsType, ElementType, VectorChannels};
use wgpu::TextureFormat;

// Implement a function that will deserialize the element type for a specific channel type
// Only supported: R, Rg, RGBA
macro_rules! impl_test {
    ($channel:ident) => {
        fn handle__formats(element: ElementType) -> TextureFormat {
            match element {
                ElementType::Eight { signed, normalized } => match (signed, normalized) {
                    (true, true) => TextureFormat::R8Snorm,
                    (true, false) => TextureFormat::R8Sint,
                    (false, true) => TextureFormat::R8Unorm,
                    (false, false) => TextureFormat::R8Uint,
                },
                ElementType::Sixteen { signed, normalized } => match (signed, normalized) {
                    (true, true) => TextureFormat::R16Snorm,
                    (true, false) => TextureFormat::R16Sint,
                    (false, true) => TextureFormat::R16Unorm,
                    (false, false) => TextureFormat::R16Uint,
                },
                ElementType::ThirtyTwo { signed } => match signed {
                    true => TextureFormat::R32Sint,
                    false => TextureFormat::R32Uint,
                },
                ElementType::FloatSixteen => TextureFormat::R16Float,
                ElementType::FloatThirtyTwo => TextureFormat::R32Float,
            }
        }
    };
}


// Converts the given vector channels to the proper format
pub const fn pick_format_from_vector_channels(
    element: ElementType,
    channels: VectorChannels,
) -> TextureFormat {
    impl_test!(r);

    
    // Handle BGRA formats
    fn handle_bgra_formats(element: ElementType) -> TextureFormat {
        todo!()
    }

    match channels {
        VectorChannels::One => handle_r_formats(element),
        //VectorChannels::Two => handle_rg_formats(element),
        //VectorChannels::Four => handle_rgba_formats(element),
        VectorChannels::FourSwizzled => handle_bgra_formats(element),
    }
}

// Converts the given depth channel to the proper format
pub const fn pick_depth_format(
    element_type: ElementType,
) -> TextureFormat {
    match element_type {
        ElementType::Sixteen {
            signed: false,
            normalized: true,
        } => TextureFormat::Depth16Unorm,
        ElementType::FloatThirtyTwo => TextureFormat::Depth32Float,
        _ => panic!(),
    }
}

// Converts the given stencil channel to the proper format
pub const fn pick_stencil_format(
    element_type: ElementType,
) -> TextureFormat {
    match element_type {
        ElementType::Eight { signed: false, normalized: false } => TextureFormat::Stencil8,
        _ => panic!()
    }    
}

// Converts the given data to the proper format
// This is called within the Texel::FORMAT and Vertex::FORMAT
pub const fn pick_format_from_params(
    element_type: ElementType,
    channels_type: ChannelsType,
) -> TextureFormat {
    match channels_type {
        ChannelsType::Vector(channels) => pick_format_from_vector_channels(element_type, channels),
        ChannelsType::Depth => pick_depth_format(element_type),
        ChannelsType::Stencil => pick_stencil_format(element_type),
    }
}

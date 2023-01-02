use crate::{BaseType, ChannelsType, VectorChannels, ElementType};
use vulkan::vk;

// Converts the given vector channels to the proper format
pub const fn pick_format_from_vector_channels(
    element_type: ElementType,
    channels: VectorChannels,
) -> vk::Format {
    // Handle normalizable integers of the given bitsize
    // Takes care of u8, i8, u16, i16
    const fn pick_normalizable_int(
        bitsize: u32,
        normalized: bool,
        signed: bool,
        channels: VectorChannels,
    ) -> vk::Format {
        let offset = match bitsize {
            8 => vk::Format::R8_UNORM.as_raw(),
            16 => vk::Format::R16_UNORM.as_raw(),
            _ => panic!(),
        };

        let signed_offset = signed as i32;

        let normalized_offset = !normalized as i32 * 4;
        let channels_offset = match channels.count() {
            1 => 0,
            2 => 7,
            3 => 14,
            4 if bitsize == 16 => 21,
            4 if bitsize != 16 => 28,
            _ => panic!(),
        };

        let raw = offset
            + signed_offset
            + normalized_offset
            + channels_offset;
        vk::Format::from_raw(raw)
    }

    // Handle non-normalizable integers of the given bitsize
    // Takes care of u32, i32, u32, i32
    const fn pick_non_normalizable_int(
        bitsize: u32,
        signed: bool,
        channels: VectorChannels,
    ) -> vk::Format {
        let offset = match bitsize {
            32 => vk::Format::R32_UINT.as_raw(),
            64 => vk::Format::R64_UINT.as_raw(),
            _ => panic!(),
        };

        let signed_offset = signed as i32;
        let channels_offset = (channels.count() as i32 - 1) * 3;
        let raw = offset + signed_offset + channels_offset;
        vk::Format::from_raw(raw)
    }

    // Handle floats of the given bitsize
    // Takes care of f16, f32, f64
    const fn pick_float(
        bitsize: u32,
        channels: VectorChannels,
    ) -> vk::Format {
        let offset = match bitsize {
            16 => vk::Format::R16_SFLOAT.as_raw(),
            32 => vk::Format::R32_SFLOAT.as_raw(),
            64 => vk::Format::R64_SFLOAT.as_raw(),
            _ => panic!(),
        };

        let channels_offset = if bitsize == 16 {
            (channels.count() as i32 - 1) * 7
        } else {
            (channels.count() as i32 - 1) * 3
        };

        let raw = offset + channels_offset;
        vk::Format::from_raw(raw)
    }

    match element_type {
        ElementType::Eight { signed, normalized } => {
            pick_normalizable_int(8, normalized, signed, channels)
        }
        ElementType::Sixteen { signed, normalized } => {
            pick_normalizable_int(16, normalized, signed, channels)
        }
        ElementType::ThirtyTwo { signed } => {
            pick_non_normalizable_int(32, signed, channels)
        }
        ElementType::SixtyFour { signed } => {
            pick_non_normalizable_int(64, signed, channels)
        }
        ElementType::FloatSixteen => pick_float(16, channels),
        ElementType::FloatThirtyTwo => pick_float(32, channels),
        ElementType::FloatSixtyFour => pick_float(64, channels),
    }
}

// Converts the given depth channel to the proper format
pub const fn pick_depth_format(element_type: ElementType) -> vk::Format {
    match element_type {
        ElementType::Sixteen {
            signed: false,
            normalized: true,
        } => vk::Format::D16_UNORM,
        ElementType::FloatThirtyTwo => vk::Format::D32_SFLOAT,
        _ => panic!(),
    }
}

// Converts the given stencil channel to the proper format
pub const fn pick_stencil_format(
    element_type: ElementType,
) -> vk::Format {
    match element_type {
        ElementType::Eight {
            signed: false,
            normalized: false,
        } => vk::Format::S8_UINT,
        _ => panic!(),
    }
}

// Converts the given data to the proper format
// This is called within the Texel::FORMAT and Vertex::FORMAT
pub const fn pick_format_from_params(
    element_type: ElementType,
    channels_type: ChannelsType,
) -> vk::Format {
    match channels_type {
        ChannelsType::Vector(channels) => {
            pick_format_from_vector_channels(element_type, channels)
        }
        ChannelsType::Depth => pick_depth_format(element_type),
        ChannelsType::Stencil => pick_stencil_format(element_type),
    }
}

use vulkan::vk;
use crate::{ChannelsType, BaseType, ColorChannels};

// Converts the given color channels to the proper format
const fn pick_format_from_color_channels(
    bits_per_channel: u32,
    base_type: BaseType,
    normalized: bool,
    channels: ColorChannels,
) -> vk::Format {
    let is_float = match base_type {
        BaseType::Float => true,
        _ => false,
    };

    if bits_per_channel <= 16 && !is_float {
        let offset = match bits_per_channel {
            8 => vk::Format::R8_UNORM.as_raw(),
            16 => vk::Format::R16_UNORM.as_raw(),
            _ => panic!()
        };

        let signed_offset = match base_type {
            BaseType::UnsignedInt => 0,
            BaseType::SignedInt => 1,
            BaseType::Float => 6,
        };

        let normalized_offset = match normalized {
            true => 0,
            false => 4,
        };
        
        let channels_offset = match channels.count() {
            1 => 0,
            2 => 7,
            3 => 14,
            4 if bits_per_channel == 16 => 21,
            4 if bits_per_channel != 16 => 28,
            _ => panic!()
        };

        let raw = offset + signed_offset + normalized_offset + channels_offset;
        vk::Format::from_raw(raw)
    } else if is_float {
        todo!()
    } else {
        todo!()
    }
}

// Converts the given data to the proper format
// This is called within the Texel::format method
pub const fn pick_format_from_params(
    bits_per_channel: u32,
    base_type: BaseType,
    normalized: bool,
    channels_type: ChannelsType
) -> vk::Format {
    match channels_type {
        ChannelsType::Color(channels) => pick_format_from_color_channels(
            bits_per_channel,
            base_type,
            normalized,
            channels
        ),
        ChannelsType::Depth => todo!(),
        ChannelsType::Stencil => todo!(),
    }
}
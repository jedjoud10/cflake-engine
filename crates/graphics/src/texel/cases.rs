use vulkan::vk;
use crate::{ChannelsType, BaseType};

// Converts the given data to the proper format
// This is called within the Texel::format method
const fn pick_from_params(
    bits_per_channel: u32,
    base_type: BaseType,
    normalized: bool,
    channels_type: ChannelsType
) -> vk::Format {
    todo!()
}
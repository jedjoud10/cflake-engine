use vulkan::vk;
use crate::{R, ChannelsType, Base, BaseType};
use super::AnyElement;

// An untyped wrapper around texel types
pub struct UntypedTexel {
    // Format related
    pub format: vk::Format,
    pub channels: ChannelsType,
    pub normalized: bool,
    pub base: BaseType,

    // Storage/memory related
    pub total_bits: u32,
    pub bits_per_channel: u32,
}

// This trait defines the layout for a single texel that will be stored within textures
// The texel format of each texture is specified at compile time
// This assumes a very simple case of multi-channel texels
pub trait Texel: 'static + Sized {
    // Number of bits per channel
    const BITS_PER_CHANNEL: u32;

    // Untyped representation of the underlying base
    // TODO: Rename this mofo
    const BASE_TYPE: BaseType;

    // Is the data normalized to it's appropriate range
    const NORMALIZED: bool;

    // Type of channels (either R, RG, RGB, RGBA, Depth, Stencil)
    const CHANNELS_TYPE: ChannelsType; 

    // Compile time Vulkan format (calls to cases::guess)
    const FORMAT: vk::Format;

    // The raw data type that we will use to access texture memory 
    type Storage;

    // Get the untyped variant of this texel
    fn untyped() -> UntypedTexel {
        UntypedTexel {
            format: Self::FORMAT,
            channels: Self::CHANNELS_TYPE,
            base: Self::BASE_TYPE,
            normalized: Self::NORMALIZED,
            bits_per_channel: Self::BITS_PER_CHANNEL,
            total_bits: Self::BITS_PER_CHANNEL * Self::CHANNELS_TYPE.count()
        }
    }
}
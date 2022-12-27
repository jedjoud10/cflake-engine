#[cfg(test)]
mod texels {
    use vulkan::vk;

    use crate::texel::*;
    
    #[test]
    fn size() {
        assert_eq!(R::<u8>::BITS_PER_CHANNEL, 8);
        assert_eq!(R::<u16>::BITS_PER_CHANNEL, 16);
        assert_eq!(R::<u32>::BITS_PER_CHANNEL, 32);
        assert_eq!(R::<u64>::BITS_PER_CHANNEL, 64);
        assert_eq!(R::<f32>::BITS_PER_CHANNEL, 32);
    }

    #[test]
    fn channels() {
        assert_eq!(R::<u8>::CHANNELS_TYPE.count(), 1);
        assert_eq!(RG::<u8>::CHANNELS_TYPE.count(), 2);
        assert_eq!(RGB::<u8>::CHANNELS_TYPE.count(), 3);
        assert_eq!(RGBA::<u8>::CHANNELS_TYPE.count(), 4);

        assert_eq!(R::<u8>::FORMAT, vk::Format::R8_UINT);
        assert_eq!(RG::<u8>::FORMAT, vk::Format::R8G8_UINT);
        assert_eq!(RGB::<u8>::FORMAT, vk::Format::R8G8B8_UINT);
        assert_eq!(RGBA::<u8>::FORMAT, vk::Format::R8G8B8A8_UINT);

        assert_eq!(R::<u16>::FORMAT, vk::Format::R16_UINT);
        assert_eq!(RG::<u16>::FORMAT, vk::Format::R16G16_UINT);
        assert_eq!(RGB::<u16>::FORMAT, vk::Format::R16G16B16_UINT);
        assert_eq!(RGBA::<u16>::FORMAT, vk::Format::R16G16B16A16_UINT);

        assert_eq!(R::<f32>::FORMAT, vk::Format::R32_SFLOAT);
        assert_eq!(RG::<f32>::FORMAT, vk::Format::R32G32_SFLOAT);
        assert_eq!(RGB::<f32>::FORMAT, vk::Format::R32G32B32_SFLOAT);
        assert_eq!(RGBA::<f32>::FORMAT, vk::Format::R32G32B32A32_SFLOAT);
    }

    #[test]
    fn normalized() {
        assert_eq!(R::<Normalized<u8>>::FORMAT, vk::Format::R8_UNORM);
        assert_eq!(RG::<Normalized<u8>>::FORMAT, vk::Format::R8G8_UNORM);
        assert_eq!(RGB::<Normalized<u8>>::FORMAT, vk::Format::R8G8B8_UNORM);
        assert_eq!(RGBA::<Normalized<u8>>::FORMAT, vk::Format::R8G8B8A8_UNORM);

        assert_eq!(R::<Normalized<u16>>::FORMAT, vk::Format::R16_UNORM);
        assert_eq!(RG::<Normalized<u16>>::FORMAT, vk::Format::R16G16_UNORM);
        assert_eq!(RGB::<Normalized<u16>>::FORMAT, vk::Format::R16G16B16_UNORM);
        assert_eq!(RGBA::<Normalized<u16>>::FORMAT, vk::Format::R16G16B16A16_UNORM);

        assert_eq!(R::<Normalized<i8>>::FORMAT, vk::Format::R8_SNORM);
        assert_eq!(RG::<Normalized<i8>>::FORMAT, vk::Format::R8G8_SNORM);
        assert_eq!(RGB::<Normalized<i8>>::FORMAT, vk::Format::R8G8B8_SNORM);
        assert_eq!(RGBA::<Normalized<i8>>::FORMAT, vk::Format::R8G8B8A8_SNORM);

        assert_eq!(R::<Normalized<i16>>::FORMAT, vk::Format::R16_SNORM);
        assert_eq!(RG::<Normalized<i16>>::FORMAT, vk::Format::R16G16_SNORM);
        assert_eq!(RGB::<Normalized<i16>>::FORMAT, vk::Format::R16G16B16_SNORM);
        assert_eq!(RGBA::<Normalized<i16>>::FORMAT, vk::Format::R16G16B16A16_SNORM);
    }

    #[test]
    fn signed() {
        type SX = R<i8>;
        type SY = R<i16>;
        type SZ = R<i32>;
    }

    #[test]
    fn unsigned() {
        type UX = R<u8>;
        type UY = R<u16>;
        type UZ = R<u32>;
    }

    #[test]
    fn special() {

    }
}
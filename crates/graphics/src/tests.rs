#[cfg(test)]
mod texels {
    use crate::texture::Texel;
    use crate::format::*;
    use half::f16;
    use vulkan::vk;

    #[test]
    fn size() {
        assert_eq!(R::<u8>::BITS_PER_CHANNEL, 8);
        assert_eq!(R::<u16>::BITS_PER_CHANNEL, 16);
        assert_eq!(R::<u32>::BITS_PER_CHANNEL, 32);
        assert_eq!(R::<u64>::BITS_PER_CHANNEL, 64);
        assert_eq!(R::<f16>::BITS_PER_CHANNEL, 16);
        assert_eq!(R::<f32>::BITS_PER_CHANNEL, 32);
        assert_eq!(R::<f64>::BITS_PER_CHANNEL, 64);
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
        assert_eq!(
            RGBA::<u16>::FORMAT,
            vk::Format::R16G16B16A16_UINT
        );

        assert_eq!(R::<u32>::FORMAT, vk::Format::R32_UINT);
        assert_eq!(RG::<u32>::FORMAT, vk::Format::R32G32_UINT);
        assert_eq!(RGB::<u32>::FORMAT, vk::Format::R32G32B32_UINT);
        assert_eq!(
            RGBA::<u32>::FORMAT,
            vk::Format::R32G32B32A32_UINT
        );

        assert_eq!(R::<u64>::FORMAT, vk::Format::R64_UINT);
        assert_eq!(RG::<u64>::FORMAT, vk::Format::R64G64_UINT);
        assert_eq!(RGB::<u64>::FORMAT, vk::Format::R64G64B64_UINT);
        assert_eq!(
            RGBA::<u64>::FORMAT,
            vk::Format::R64G64B64A64_UINT
        );
    }

    #[test]
    fn float() {
        assert_eq!(R::<f16>::FORMAT, vk::Format::R16_SFLOAT);
        assert_eq!(RG::<f16>::FORMAT, vk::Format::R16G16_SFLOAT);
        assert_eq!(RGB::<f16>::FORMAT, vk::Format::R16G16B16_SFLOAT);
        assert_eq!(
            RGBA::<f16>::FORMAT,
            vk::Format::R16G16B16A16_SFLOAT
        );

        assert_eq!(R::<f32>::FORMAT, vk::Format::R32_SFLOAT);
        assert_eq!(RG::<f32>::FORMAT, vk::Format::R32G32_SFLOAT);
        assert_eq!(RGB::<f32>::FORMAT, vk::Format::R32G32B32_SFLOAT);
        assert_eq!(
            RGBA::<f32>::FORMAT,
            vk::Format::R32G32B32A32_SFLOAT
        );

        assert_eq!(R::<f64>::FORMAT, vk::Format::R64_SFLOAT);
        assert_eq!(RG::<f64>::FORMAT, vk::Format::R64G64_SFLOAT);
        assert_eq!(RGB::<f64>::FORMAT, vk::Format::R64G64B64_SFLOAT);
        assert_eq!(
            RGBA::<f64>::FORMAT,
            vk::Format::R64G64B64A64_SFLOAT
        );
    }

    #[test]
    fn normalized() {
        assert_eq!(R::<Normalized<u8>>::FORMAT, vk::Format::R8_UNORM);
        assert_eq!(
            RG::<Normalized<u8>>::FORMAT,
            vk::Format::R8G8_UNORM
        );
        assert_eq!(
            RGB::<Normalized<u8>>::FORMAT,
            vk::Format::R8G8B8_UNORM
        );
        assert_eq!(
            RGBA::<Normalized<u8>>::FORMAT,
            vk::Format::R8G8B8A8_UNORM
        );

        assert_eq!(
            R::<Normalized<u16>>::FORMAT,
            vk::Format::R16_UNORM
        );
        assert_eq!(
            RG::<Normalized<u16>>::FORMAT,
            vk::Format::R16G16_UNORM
        );
        assert_eq!(
            RGB::<Normalized<u16>>::FORMAT,
            vk::Format::R16G16B16_UNORM
        );
        assert_eq!(
            RGBA::<Normalized<u16>>::FORMAT,
            vk::Format::R16G16B16A16_UNORM
        );

        assert_eq!(R::<Normalized<i8>>::FORMAT, vk::Format::R8_SNORM);
        assert_eq!(
            RG::<Normalized<i8>>::FORMAT,
            vk::Format::R8G8_SNORM
        );
        assert_eq!(
            RGB::<Normalized<i8>>::FORMAT,
            vk::Format::R8G8B8_SNORM
        );
        assert_eq!(
            RGBA::<Normalized<i8>>::FORMAT,
            vk::Format::R8G8B8A8_SNORM
        );

        assert_eq!(
            R::<Normalized<i16>>::FORMAT,
            vk::Format::R16_SNORM
        );
        assert_eq!(
            RG::<Normalized<i16>>::FORMAT,
            vk::Format::R16G16_SNORM
        );
        assert_eq!(
            RGB::<Normalized<i16>>::FORMAT,
            vk::Format::R16G16B16_SNORM
        );
        assert_eq!(
            RGBA::<Normalized<i16>>::FORMAT,
            vk::Format::R16G16B16A16_SNORM
        );
    }

    #[test]
    fn signed() {
        assert_eq!(R::<i8>::FORMAT, vk::Format::R8_SINT);
        assert_eq!(RG::<i8>::FORMAT, vk::Format::R8G8_SINT);
        assert_eq!(RGB::<i8>::FORMAT, vk::Format::R8G8B8_SINT);
        assert_eq!(RGBA::<i8>::FORMAT, vk::Format::R8G8B8A8_SINT);

        assert_eq!(R::<i16>::FORMAT, vk::Format::R16_SINT);
        assert_eq!(RG::<i16>::FORMAT, vk::Format::R16G16_SINT);
        assert_eq!(RGB::<i16>::FORMAT, vk::Format::R16G16B16_SINT);
        assert_eq!(
            RGBA::<i16>::FORMAT,
            vk::Format::R16G16B16A16_SINT
        );
    }

    #[test]
    fn unsigned() {
        assert_eq!(R::<u8>::FORMAT, vk::Format::R8_UINT);
        assert_eq!(RG::<u8>::FORMAT, vk::Format::R8G8_UINT);
        assert_eq!(RGB::<u8>::FORMAT, vk::Format::R8G8B8_UINT);
        assert_eq!(RGBA::<u8>::FORMAT, vk::Format::R8G8B8A8_UINT);

        assert_eq!(R::<u16>::FORMAT, vk::Format::R16_UINT);
        assert_eq!(RG::<u16>::FORMAT, vk::Format::R16G16_UINT);
        assert_eq!(RGB::<u16>::FORMAT, vk::Format::R16G16B16_UINT);
        assert_eq!(
            RGBA::<u16>::FORMAT,
            vk::Format::R16G16B16A16_UINT
        );

        assert_eq!(R::<u32>::FORMAT, vk::Format::R32_UINT);
        assert_eq!(RG::<u32>::FORMAT, vk::Format::R32G32_UINT);
        assert_eq!(RGB::<u32>::FORMAT, vk::Format::R32G32B32_UINT);
        assert_eq!(
            RGBA::<u32>::FORMAT,
            vk::Format::R32G32B32A32_UINT
        );

        assert_eq!(R::<u64>::FORMAT, vk::Format::R64_UINT);
        assert_eq!(RG::<u64>::FORMAT, vk::Format::R64G64_UINT);
        assert_eq!(RGB::<u64>::FORMAT, vk::Format::R64G64B64_UINT);
        assert_eq!(
            RGBA::<u64>::FORMAT,
            vk::Format::R64G64B64A64_UINT
        );
    }

    #[test]
    fn special() {
        assert_eq!(
            Depth::<Normalized<u16>>::FORMAT,
            vk::Format::D16_UNORM
        );
        assert_eq!(Depth::<f32>::FORMAT, vk::Format::D32_SFLOAT);
        assert_eq!(Stencil::<u8>::FORMAT, vk::Format::S8_UINT);
    }
}

#[cfg(test)]
mod vertex {
    use crate::pipeline::Vertex;
    use crate::format::*;
    use half::f16;
    use vulkan::vk;

    #[test]
    fn positional() {
        assert_eq!(
            XYZ::<f16>::FORMAT,
            vk::Format::R16G16B16_SFLOAT,
        );

        assert_eq!(
            XYZ::<f32>::FORMAT,
            vk::Format::R32G32B32_SFLOAT,
        );

        assert_eq!(
            XYZ::<f64>::FORMAT,
            vk::Format::R64G64B64_SFLOAT,
        );

        assert_eq!(
            XYZ::<f32>::FORMAT,
            vk::Format::R32G32B32_SFLOAT,
        );
    }

    #[test]
    fn normals() {
        assert_eq!(
            XYZ::<Normalized<i8>>::FORMAT,
            vk::Format::R8G8B8_SNORM,
        );

        assert_eq!(
            XYZ::<Normalized<i16>>::FORMAT,
            vk::Format::R16G16B16_SNORM,
        );
    }

    #[test]
    fn uvs() {
        assert_eq!(
            XY::<Normalized<u8>>::FORMAT,
            vk::Format::R8G8_UNORM,
        );

        assert_eq!(
            XY::<Normalized<u16>>::FORMAT,
            vk::Format::R16G16_UNORM,
        );
    }
}
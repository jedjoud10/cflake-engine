#[cfg(test)]
mod texels {
    use crate::format::*;
    use crate::texture::Texel;
    use half::f16;
    use wgpu::TextureFormat;

    #[test]
    fn size() {
        assert_eq!(R::<u8>::BITS_PER_CHANNEL, 8);
        assert_eq!(R::<u16>::BITS_PER_CHANNEL, 16);
        assert_eq!(R::<u32>::BITS_PER_CHANNEL, 32);
        assert_eq!(R::<f16>::BITS_PER_CHANNEL, 16);
        assert_eq!(R::<f32>::BITS_PER_CHANNEL, 32);
        assert_eq!(R::<Normalized<u8>>::BITS_PER_CHANNEL, 8);
        assert_eq!(R::<Normalized<u16>>::BITS_PER_CHANNEL, 16);
    }

    #[test]
    fn channels() {
        assert_eq!(R::<u8>::CHANNELS_TYPE.count(), 1);
        assert_eq!(RG::<u8>::CHANNELS_TYPE.count(), 2);
        assert_eq!(RGBA::<u8>::CHANNELS_TYPE.count(), 4);

        assert_eq!(R::<u8>::FORMAT, TextureFormat::R8Uint);
        assert_eq!(RG::<u8>::FORMAT, TextureFormat::Rg8Uint);
        assert_eq!(RGBA::<u8>::FORMAT, TextureFormat::Rgba8Uint);

        assert_eq!(R::<u16>::FORMAT, TextureFormat::R16Uint);
        assert_eq!(RG::<u16>::FORMAT, TextureFormat::Rg16Uint);
        assert_eq!(
            RGBA::<u16>::FORMAT,
            TextureFormat::Rgba16Uint
        );

        assert_eq!(R::<u32>::FORMAT, TextureFormat::R32Uint);
        assert_eq!(RG::<u32>::FORMAT, TextureFormat::Rg32Uint);
        assert_eq!(
            RGBA::<u32>::FORMAT,
            TextureFormat::Rgba32Uint
        );
    }

    #[test]
    fn float() {
        assert_eq!(R::<f16>::FORMAT, TextureFormat::R16Float);
        assert_eq!(RG::<f16>::FORMAT, TextureFormat::Rg16Float);
        assert_eq!(
            RGBA::<f16>::FORMAT,
            TextureFormat::Rgba16Float
        );

        assert_eq!(R::<f32>::FORMAT, TextureFormat::R32Float);
        assert_eq!(RG::<f32>::FORMAT, TextureFormat::Rg32Float);
        assert_eq!(
            RGBA::<f32>::FORMAT,
            TextureFormat::Rgba32Float
        );
    }

    #[test]
    fn normalized() {
        assert_eq!(R::<Normalized<u8>>::FORMAT, TextureFormat::R8Unorm);
        assert_eq!(
            RG::<Normalized<u8>>::FORMAT,
            TextureFormat::Rg8Unorm
        );
        assert_eq!(
            RGBA::<Normalized<u8>>::FORMAT,
            TextureFormat::Rgba8Unorm
        );
        assert_eq!(
            BGRA::<Normalized<u8>>::FORMAT,
            TextureFormat::Bgra8Unorm
        );

        assert_eq!(
            R::<Normalized<u16>>::FORMAT,
            TextureFormat::R8Unorm
        );
        assert_eq!(
            RG::<Normalized<u16>>::FORMAT,
            TextureFormat::Rg8Unorm
        );
        assert_eq!(
            RGBA::<Normalized<u16>>::FORMAT,
            TextureFormat::Rgba8Unorm
        );

        assert_eq!(R::<Normalized<i8>>::FORMAT, TextureFormat::R8Snorm);
        assert_eq!(
            RG::<Normalized<i8>>::FORMAT,
            TextureFormat::Rg8Snorm
        );
        assert_eq!(
            RGBA::<Normalized<i8>>::FORMAT,
            TextureFormat::Rgba8Snorm
        );

        /*
        assert_eq!(
            BGRA::<Normalized<i8>>::FORMAT,
            TextureFormat::Bgra8
        );
        */

        assert_eq!(
            R::<Normalized<i16>>::FORMAT,
            TextureFormat::R16Snorm
        );
        assert_eq!(
            RG::<Normalized<i16>>::FORMAT,
            TextureFormat::Rg16Snorm
        );
        assert_eq!(
            RGBA::<Normalized<i16>>::FORMAT,
            TextureFormat::Rgba16Snorm
        );
    }

    #[test]
    fn signed() {
        assert_eq!(R::<i8>::FORMAT, TextureFormat::R8Sint);
        assert_eq!(RG::<i8>::FORMAT, TextureFormat::Rg8Sint);
        assert_eq!(RGBA::<i8>::FORMAT, TextureFormat::Rgba8Sint);

        assert_eq!(R::<i16>::FORMAT, TextureFormat::R16Sint);
        assert_eq!(RG::<i16>::FORMAT, TextureFormat::Rg16Sint);
        assert_eq!(
            RGBA::<i16>::FORMAT,
            TextureFormat::Rgba16Sint
        );
    }

    #[test]
    fn unsigned() {
        assert_eq!(R::<u8>::FORMAT, TextureFormat::R8Uint);
        assert_eq!(RG::<u8>::FORMAT, TextureFormat::Rg8Uint);
        assert_eq!(RGBA::<u8>::FORMAT, TextureFormat::Rgba8Uint);

        assert_eq!(R::<u16>::FORMAT, TextureFormat::R16Uint);
        assert_eq!(RG::<u16>::FORMAT, TextureFormat::Rg16Uint);
        assert_eq!(
            RGBA::<u16>::FORMAT,
            TextureFormat::Rgba16Uint
        );

        assert_eq!(R::<u32>::FORMAT, TextureFormat::Rgba32Uint);
        assert_eq!(RG::<u32>::FORMAT, TextureFormat::Rg32Uint);
        assert_eq!(
            RGBA::<u32>::FORMAT,
            TextureFormat::Rgba32Uint
        );
    }

    #[test]
    fn special() {
        assert_eq!(
            Depth::<Normalized<u16>>::FORMAT,
            TextureFormat::Depth16Unorm
        );
        assert_eq!(Depth::<f32>::FORMAT, TextureFormat::Depth32Float);
        assert_eq!(Stencil::<u8>::FORMAT, TextureFormat::Stencil8);
    }
}

/*
#[cfg(test)]
mod vertex {
    use crate::format::*;
    use crate::pipeline::Vertex;
    use half::f16;
    use crate::vulkan::vk;

    #[test]
    fn positional() {
        assert_eq!(XYZ::<f16>::FORMAT, TextureFormat::R16G16B16_SFLOAT,);
        assert_eq!(XYZ::<f32>::FORMAT, TextureFormat::R32G32B32_SFLOAT,);
        assert_eq!(XYZ::<f64>::FORMAT, TextureFormat::R64G64B64_SFLOAT,);
        assert_eq!(XYZ::<f32>::FORMAT, TextureFormat::R32G32B32_SFLOAT,);
    }

    #[test]
    fn normals() {
        assert_eq!(
            XYZ::<Normalized<i8>>::FORMAT,
            TextureFormat::R8G8B8_SNORM,
        );

        assert_eq!(
            XYZ::<Normalized<i16>>::FORMAT,
            TextureFormat::R16G16B16_SNORM,
        );
    }

    #[test]
    fn uvs() {
        assert_eq!(
            XY::<Normalized<u8>>::FORMAT,
            TextureFormat::R8G8_UNORM,
        );

        assert_eq!(
            XY::<Normalized<u16>>::FORMAT,
            TextureFormat::R16G16_UNORM,
        );
    }
}
*/
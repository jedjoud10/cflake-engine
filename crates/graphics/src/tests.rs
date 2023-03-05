#[cfg(test)]
mod texels {
    use crate::format::*;
    use half::f16;
    use wgpu::TextureFormat;

    #[test]
    fn size() {
        assert_eq!(R::<u8>::bytes_per_channel(), 1);
        assert_eq!(R::<u16>::bytes_per_channel(), 2);
        assert_eq!(R::<u32>::bytes_per_channel(), 4);
        assert_eq!(R::<f16>::bytes_per_channel(), 2);
        assert_eq!(R::<f32>::bytes_per_channel(), 4);
        assert_eq!(R::<Normalized<u8>>::bytes_per_channel(), 1);
        assert_eq!(R::<Normalized<u16>>::bytes_per_channel(), 2);
    }

    #[test]
    fn channels() {
        assert_eq!(R::<u8>::channels().count(), 1);
        assert_eq!(RG::<u8>::channels().count(), 2);
        assert_eq!(RGBA::<u8>::channels().count(), 4);

        assert_eq!(R::<u8>::format(), TextureFormat::R8Uint);
        assert_eq!(RG::<u8>::format(), TextureFormat::Rg8Uint);
        assert_eq!(RGBA::<u8>::format(), TextureFormat::Rgba8Uint);

        assert_eq!(R::<u16>::format(), TextureFormat::R16Uint);
        assert_eq!(RG::<u16>::format(), TextureFormat::Rg16Uint);
        assert_eq!(RGBA::<u16>::format(), TextureFormat::Rgba16Uint);

        assert_eq!(R::<u32>::format(), TextureFormat::R32Uint);
        assert_eq!(RG::<u32>::format(), TextureFormat::Rg32Uint);
        assert_eq!(RGBA::<u32>::format(), TextureFormat::Rgba32Uint);
    }

    #[test]
    fn float() {
        assert_eq!(R::<f16>::format(), TextureFormat::R16Float);
        assert_eq!(RG::<f16>::format(), TextureFormat::Rg16Float);
        assert_eq!(RGBA::<f16>::format(), TextureFormat::Rgba16Float);

        assert_eq!(R::<f32>::format(), TextureFormat::R32Float);
        assert_eq!(RG::<f32>::format(), TextureFormat::Rg32Float);
        assert_eq!(RGBA::<f32>::format(), TextureFormat::Rgba32Float);
    }

    #[test]
    fn normalized() {
        assert_eq!(
            R::<Normalized<u8>>::format(),
            TextureFormat::R8Unorm
        );
        assert_eq!(
            RG::<Normalized<u8>>::format(),
            TextureFormat::Rg8Unorm
        );
        assert_eq!(
            RGBA::<Normalized<u8>>::format(),
            TextureFormat::Rgba8Unorm
        );
        assert_eq!(
            BGRA::<Normalized<u8>>::format(),
            TextureFormat::Bgra8Unorm
        );

        assert_eq!(
            R::<Normalized<u16>>::format(),
            TextureFormat::R16Unorm
        );
        assert_eq!(
            RG::<Normalized<u16>>::format(),
            TextureFormat::Rg16Unorm
        );
        assert_eq!(
            RGBA::<Normalized<u16>>::format(),
            TextureFormat::Rgba16Unorm
        );

        assert_eq!(
            R::<Normalized<i8>>::format(),
            TextureFormat::R8Snorm
        );
        assert_eq!(
            RG::<Normalized<i8>>::format(),
            TextureFormat::Rg8Snorm
        );
        assert_eq!(
            RGBA::<Normalized<i8>>::format(),
            TextureFormat::Rgba8Snorm
        );

        assert_eq!(
            R::<Normalized<i16>>::format(),
            TextureFormat::R16Snorm
        );
        assert_eq!(
            RG::<Normalized<i16>>::format(),
            TextureFormat::Rg16Snorm
        );
        assert_eq!(
            RGBA::<Normalized<i16>>::format(),
            TextureFormat::Rgba16Snorm
        );
    }

    #[test]
    fn signed() {
        assert_eq!(R::<i8>::format(), TextureFormat::R8Sint);
        assert_eq!(RG::<i8>::format(), TextureFormat::Rg8Sint);
        assert_eq!(RGBA::<i8>::format(), TextureFormat::Rgba8Sint);

        assert_eq!(R::<i16>::format(), TextureFormat::R16Sint);
        assert_eq!(RG::<i16>::format(), TextureFormat::Rg16Sint);
        assert_eq!(RGBA::<i16>::format(), TextureFormat::Rgba16Sint);
    }

    #[test]
    fn unsigned() {
        assert_eq!(R::<u8>::format(), TextureFormat::R8Uint);
        assert_eq!(RG::<u8>::format(), TextureFormat::Rg8Uint);
        assert_eq!(RGBA::<u8>::format(), TextureFormat::Rgba8Uint);

        assert_eq!(R::<u16>::format(), TextureFormat::R16Uint);
        assert_eq!(RG::<u16>::format(), TextureFormat::Rg16Uint);
        assert_eq!(RGBA::<u16>::format(), TextureFormat::Rgba16Uint);

        assert_eq!(R::<u32>::format(), TextureFormat::R32Uint);
        assert_eq!(RG::<u32>::format(), TextureFormat::Rg32Uint);
        assert_eq!(RGBA::<u32>::format(), TextureFormat::Rgba32Uint);
    }

    #[test]
    fn special() {
        assert_eq!(
            Depth::<Normalized<u16>>::format(),
            TextureFormat::Depth16Unorm
        );
        assert_eq!(
            Depth::<f32>::format(),
            TextureFormat::Depth32Float
        );
        assert_eq!(Stencil::<u8>::format(), TextureFormat::Stencil8);
    }
}

#[cfg(test)]
mod color {
    use crate::format::*;
    use half::f16;
    use wgpu::TextureFormat;

    #[test]
    fn convert_unsigned() {
        assert_eq!(R::<u8>::into_target(u8::MIN).x, u8::MIN as f32);
        assert_eq!(R::<u8>::into_target(u8::MAX).x, u8::MAX as f32);
        assert_eq!(
            R::<u16>::into_target(u16::MIN).x,
            u16::MIN as f32
        );
        assert_eq!(
            R::<u16>::into_target(u16::MAX).x,
            u16::MAX as f32
        );
        assert_eq!(
            R::<u32>::into_target(u32::MIN).x,
            u32::MIN as f32
        );
        assert_eq!(
            R::<u32>::into_target(u32::MAX).x,
            u32::MAX as f32
        );

        assert_eq!(
            RG::<u8>::into_target(vek::Vec2::broadcast(u8::MIN)).xy(),
            vek::Vec2::broadcast(u8::MIN as f32)
        );
        assert_eq!(
            RG::<u8>::into_target(vek::Vec2::broadcast(u8::MAX)).xy(),
            vek::Vec2::broadcast(u8::MAX as f32)
        );
        assert_eq!(
            RG::<u16>::into_target(vek::Vec2::broadcast(u16::MIN))
                .xy(),
            vek::Vec2::broadcast(u16::MIN as f32)
        );
        assert_eq!(
            RG::<u16>::into_target(vek::Vec2::broadcast(u16::MAX))
                .xy(),
            vek::Vec2::broadcast(u16::MAX as f32)
        );
        assert_eq!(
            RG::<u32>::into_target(vek::Vec2::broadcast(u32::MIN))
                .xy(),
            vek::Vec2::broadcast(u32::MIN as f32)
        );
        assert_eq!(
            RG::<u32>::into_target(vek::Vec2::broadcast(u32::MAX))
                .xy(),
            vek::Vec2::broadcast(u32::MAX as f32)
        );

        assert_eq!(
            RGBA::<u8>::into_target(vek::Vec4::broadcast(u8::MIN)),
            vek::Vec4::broadcast(u8::MIN as f32)
        );
        assert_eq!(
            RGBA::<u8>::into_target(vek::Vec4::broadcast(u8::MAX)),
            vek::Vec4::broadcast(u8::MAX as f32)
        );
        assert_eq!(
            RGBA::<u16>::into_target(vek::Vec4::broadcast(u16::MIN)),
            vek::Vec4::broadcast(u16::MIN as f32)
        );
        assert_eq!(
            RGBA::<u16>::into_target(vek::Vec4::broadcast(u16::MAX)),
            vek::Vec4::broadcast(u16::MAX as f32)
        );
        assert_eq!(
            RGBA::<u32>::into_target(vek::Vec4::broadcast(u32::MIN)),
            vek::Vec4::broadcast(u32::MIN as f32)
        );
        assert_eq!(
            RGBA::<u32>::into_target(vek::Vec4::broadcast(u32::MAX)),
            vek::Vec4::broadcast(u32::MAX as f32)
        );
    }

    #[test]
    fn convert_signed() {
        assert_eq!(R::<i8>::into_target(i8::MIN).x, i8::MIN as f32);
        assert_eq!(R::<i8>::into_target(i8::MAX).x, i8::MAX as f32);
        assert_eq!(
            R::<i16>::into_target(i16::MIN).x,
            i16::MIN as f32
        );
        assert_eq!(
            R::<i16>::into_target(i16::MAX).x,
            i16::MAX as f32
        );
        assert_eq!(
            R::<i32>::into_target(i32::MIN).x,
            i32::MIN as f32
        );
        assert_eq!(
            R::<i32>::into_target(i32::MAX).x,
            i32::MAX as f32
        );

        assert_eq!(
            RG::<i8>::into_target(vek::Vec2::broadcast(i8::MIN)).xy(),
            vek::Vec2::broadcast(i8::MIN as f32)
        );
        assert_eq!(
            RG::<i8>::into_target(vek::Vec2::broadcast(i8::MAX)).xy(),
            vek::Vec2::broadcast(i8::MAX as f32)
        );
        assert_eq!(
            RG::<i16>::into_target(vek::Vec2::broadcast(i16::MIN))
                .xy(),
            vek::Vec2::broadcast(i16::MIN as f32)
        );
        assert_eq!(
            RG::<i16>::into_target(vek::Vec2::broadcast(i16::MAX))
                .xy(),
            vek::Vec2::broadcast(i16::MAX as f32)
        );
        assert_eq!(
            RG::<i32>::into_target(vek::Vec2::broadcast(i32::MIN))
                .xy(),
            vek::Vec2::broadcast(i32::MIN as f32)
        );
        assert_eq!(
            RG::<i32>::into_target(vek::Vec2::broadcast(i32::MAX))
                .xy(),
            vek::Vec2::broadcast(i32::MAX as f32)
        );

        assert_eq!(
            RGBA::<i8>::into_target(vek::Vec4::broadcast(i8::MIN)),
            vek::Vec4::broadcast(i8::MIN as f32)
        );
        assert_eq!(
            RGBA::<i8>::into_target(vek::Vec4::broadcast(i8::MAX)),
            vek::Vec4::broadcast(i8::MAX as f32)
        );
        assert_eq!(
            RGBA::<i16>::into_target(vek::Vec4::broadcast(i16::MIN)),
            vek::Vec4::broadcast(i16::MIN as f32)
        );
        assert_eq!(
            RGBA::<i16>::into_target(vek::Vec4::broadcast(i16::MAX)),
            vek::Vec4::broadcast(i16::MAX as f32)
        );
        assert_eq!(
            RGBA::<i32>::into_target(vek::Vec4::broadcast(i32::MIN)),
            vek::Vec4::broadcast(i32::MIN as f32)
        );
        assert_eq!(
            RGBA::<i32>::into_target(vek::Vec4::broadcast(i32::MAX)),
            vek::Vec4::broadcast(i32::MAX as f32)
        );
    }

    #[test]
    fn convert_float() {
        assert_eq!(
            R::<f16>::into_target(f16::MIN).x,
            f16::MIN.to_f32()
        );
        assert_eq!(
            R::<f16>::into_target(f16::MAX).x,
            f16::MAX.to_f32()
        );

        assert_eq!(R::<f32>::into_target(f32::MIN).x, f32::MIN);
        assert_eq!(R::<f32>::into_target(f32::MAX).x, f32::MAX);

        assert_eq!(
            RG::<f16>::into_target(vek::Vec2::broadcast(f16::MIN))
                .xy(),
            vek::Vec2::broadcast(f16::MIN.to_f32())
        );
        assert_eq!(
            RG::<f16>::into_target(vek::Vec2::broadcast(f16::MAX))
                .xy(),
            vek::Vec2::broadcast(f16::MAX.to_f32())
        );

        assert_eq!(
            RG::<f32>::into_target(vek::Vec2::broadcast(f32::MIN))
                .xy(),
            vek::Vec2::broadcast(f32::MIN)
        );
        assert_eq!(
            RG::<f32>::into_target(vek::Vec2::broadcast(f32::MAX))
                .xy(),
            vek::Vec2::broadcast(f32::MAX)
        );

        assert_eq!(
            RGBA::<f16>::into_target(vek::Vec4::broadcast(f16::MIN)),
            vek::Vec4::broadcast(f16::MIN.to_f32())
        );
        assert_eq!(
            RGBA::<f16>::into_target(vek::Vec4::broadcast(f16::MAX)),
            vek::Vec4::broadcast(f16::MAX.to_f32())
        );

        assert_eq!(
            RGBA::<f32>::into_target(vek::Vec4::broadcast(f32::MIN)),
            vek::Vec4::broadcast(f32::MIN)
        );
        assert_eq!(
            RGBA::<f32>::into_target(vek::Vec4::broadcast(f32::MAX)),
            vek::Vec4::broadcast(f32::MAX)
        );
    }
}

#[cfg(test)]
mod vertex {
    use crate::format::*;
    use half::f16;
    use wgpu::VertexFormat;

    #[test]
    fn positional() {
        assert_eq!(XYZ::<f32>::format(), VertexFormat::Float32x3);
    }

    #[test]
    fn normals() {
        assert_eq!(
            XYZW::<Normalized<i8>>::format(),
            VertexFormat::Snorm8x4,
        );

        assert_eq!(
            XYZW::<Normalized<i16>>::format(),
            VertexFormat::Snorm16x4,
        );
    }

    #[test]
    fn uvs() {
        assert_eq!(
            XY::<Normalized<u16>>::format(),
            VertexFormat::Unorm16x2,
        );
    }
}

#[cfg(test)]
mod region {
    use crate::Extent;

    #[test]
    fn levels() {
        fn broadcast(x: u32) -> vek::Extent2<u32> {
            vek::Extent2::broadcast(x)
        }

        fn levels(x: u32) -> u32 {
            broadcast(x).levels().unwrap().get() as u32
        }

        assert_eq!(levels(1), 1);
        assert_eq!(levels(2), 2);
        assert_eq!(levels(4), 3);
        assert_eq!(levels(8), 4);
        assert_eq!(levels(16), 5);
        assert_eq!(levels(32), 6);
        assert_eq!(levels(64), 7);
        assert_eq!(levels(128), 8);
        assert_eq!(levels(256), 9);
        assert_eq!(levels(512), 10);
        assert_eq!(levels(1024), 11);
        assert_eq!(levels(2048), 12);
    }
}

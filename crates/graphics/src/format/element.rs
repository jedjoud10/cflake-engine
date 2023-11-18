use std::any::TypeId;

use crate::{Base, CompressionType, GpuPod};

// Elements are just values that can be stored within channels, like u32, Normalized<i8> or i8
pub trait AnyElement: 'static {
    // Raw data representation that will be sent to the GPU
    type Storage: Base;

    // Untyped element type of AnyElement
    const ELEMENT_TYPE: ElementType;
}

impl<T: Base> AnyElement for T {
    type Storage = T;

    const ELEMENT_TYPE: ElementType = <T as Base>::ELEMENT_TYPE;
}

// Untyped element type that will be used to fetch the WGPU TextureFormat
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ElementType {
    // Integer types
    Eight { signed: bool, normalized: bool },
    Sixteen { signed: bool, normalized: bool },
    ThirtyTwo { signed: bool },

    // Floating point types
    FloatSixteen,
    FloatThirtyTwo,

    // Compressed element type, only to be used with compressed textures
    Compressed(CompressionType),
}

// This trait represents bases that can be normalized
pub trait Normalizable: AnyElement {}

impl Normalizable for i8 {}
impl Normalizable for u8 {}
impl Normalizable for i16 {}
impl Normalizable for u16 {}

// A normalized texel limiter that will the texture that the integer must be accessed as a floating point value, and that it must be in
//  the -1 - 1 range if it's a signed integer and the 0 - 1 range if it's an unsigned integer
#[derive(Default, Copy, Clone)]
pub struct Normalized<T: Normalizable>(T);

impl<T: Normalizable> AnyElement for Normalized<T> {
    type Storage = T::Storage;

    const ELEMENT_TYPE: ElementType = match <T as AnyElement>::ELEMENT_TYPE {
        ElementType::Eight { signed, .. } => ElementType::Eight {
            signed,
            normalized: true,
        },
        ElementType::Sixteen { signed, .. } => ElementType::Sixteen {
            signed,
            normalized: true,
        },
        ElementType::Compressed(x) => ElementType::Compressed(x),

        // Due to type safety (trait bound Normalizable), this panic will never be able to occur
        _ => panic!(),
    };
}

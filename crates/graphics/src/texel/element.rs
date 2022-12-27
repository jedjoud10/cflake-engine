use std::marker::PhantomData;
use crate::{Base, BaseType};

// Elements are just values that can be stored within channels, like u32, Normalized<i8> or i8
pub trait AnyElement: 'static {
    const BASE_TYPE: BaseType; 
    const ELEMENT_TYPE: ElementType;
}
impl<T: Base> AnyElement for T {
    const BASE_TYPE: BaseType = T::TYPE;
    const ELEMENT_TYPE: ElementType = match Self::BASE_TYPE {
        BaseType::Eight => ElementType::Eight { signed: T::SIGNED, normalized: false },
        BaseType::Sixteen => ElementType::Sixteen { signed: T::SIGNED, normalized: false },
        BaseType::ThirtyTwo => ElementType::ThirtyTwo { signed: T::SIGNED },
        BaseType::SixtyFour => ElementType::SixtyFour { signed: T::SIGNED },
        BaseType::FloatSixteen => ElementType::FloatSixteen,
        BaseType::FloatThirtyTwo => ElementType::FloatThirtyTwo,
        BaseType::FloatSixtyFour => ElementType::FloatSixtyFour,
    };
}

// Untyped element type
pub enum ElementType {
    Eight {
        signed: bool,
        normalized: bool
    },

    Sixteen {
        signed: bool,
        normalized: bool
    },

    ThirtyTwo {
        signed: bool,
    },

    SixtyFour {
        signed: bool,
    },

    FloatSixteen,
    FloatThirtyTwo,
    FloatSixtyFour,
}


// This trait represents bases that can be normalized
pub trait Normalizable: Base {}
impl Normalizable for i8 {}
impl Normalizable for u8 {}
impl Normalizable for i16 {}
impl Normalizable for u16 {}

// A normalized texel limiter that will the texture that the integer must be accessed as a floating point value, and that it must be in
//  the -1 - 1 range if it's a signed integer and the 0 - 1 range if it's an unsigned integer
pub struct Normalized<T: Base + Normalizable>(T);
impl<T: Base + Normalizable> AnyElement for Normalized<T> {
    const BASE_TYPE: BaseType = T::TYPE;
    const ELEMENT_TYPE: ElementType = match Self::BASE_TYPE {
        BaseType::Eight => ElementType::Eight { signed: T::SIGNED, normalized: true },
        BaseType::Sixteen => ElementType::Sixteen { signed: T::SIGNED, normalized: true },

        // Not supported by VkFormat
        _ => panic!(),
    };
}
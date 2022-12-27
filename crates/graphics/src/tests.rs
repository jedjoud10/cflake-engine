#[cfg(test)]
mod texels {
    use vulkan::vk;

    use crate::texel::*;
    
    fn channels() {
        type A = R<u8>;
        type B = RG<u8>;
        type C = RGB<u8>;
        type D = RGBA<u8>;
    }

    fn normalized() {}

    fn signed() {
        type SX = R<i8>;
        type SY = R<i16>;
        type SZ = R<i32>;
    }

    fn unsigned() {
        type UX = R<u8>;
        type UY = R<u16>;
        type UZ = R<u32>;
    }
}
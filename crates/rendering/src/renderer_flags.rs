use bitflags::bitflags;
// Yup
bitflags! {
    pub struct RendererFlags: u8 {
        const WIREFRAME = 0b00000010;
        const DEFAULT = Self::WIREFRAME.bits;
    }
}
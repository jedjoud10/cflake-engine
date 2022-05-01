use bitflags::bitflags;
// The mask that depicts what type of attributes we have enabled (other than the position attribute)
bitflags! {
    pub struct EnabledAttributes: u8 {
        const NORMAL = 1;
        const TANGENT = 1 << 1;
        const TEXCOORD = 1 << 2;
        const COLOR = 1 << 3;
    }
}

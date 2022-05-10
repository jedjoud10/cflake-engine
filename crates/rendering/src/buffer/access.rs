// Buffer access flags that specify how we will modify the buffer
bitflags::bitflags! {
    pub struct BufferAccess: u8 {
        const READ = 1;
        const WRITE = 1 << 1;
        const DYNAMIC = 1 << 2;
    }
}

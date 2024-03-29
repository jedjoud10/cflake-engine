bitflags::bitflags! {
    // How exactly are we going to use the texture?
    pub struct TextureUsage: u8 {
        // This texture will be sampled within a shader
        const SAMPLED = 1;

        // This texture will be used as a render target attachment
        const TARGET = 2 | Self::COPY_SRC.bits;

        // This texture will be used as a storage texture that we can read/write to within shaders
        const STORAGE = 4;

        // Data can be copied from the texture on the GPU side
        const COPY_SRC = 8;

        // Data can be copied into the texture on the GPU side
        // Required by the texture when we have pre-initialized data
        const COPY_DST = 16;

        // The texture can be used for reading GPU data back
        // Texture init will fail if this is set and Self::COPY_SRC is not set
        const READ = 32 | Self::COPY_SRC.bits;

        // The texture can be used to send data to the GPU
        // Texture init will fail if this is set and Self::COPY_DST is not set
        const WRITE = 64 | Self::COPY_DST.bits;
    }
}

impl Default for TextureUsage {
    fn default() -> Self {
        Self::READ | Self::COPY_DST | Self::SAMPLED
    }
}

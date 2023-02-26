// Some settings that tell us how exactly we should create the texture
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TextureMode {
    // Dynamic textures can be modified throughout their lifetime, but they cannot be resized
    Dynamic,

    // Resizable textures are just dynamic textures that we can resize
    #[default]
    Resizable,
}

bitflags::bitflags! {
    // How exactly are we going to use the texture?
    pub struct TextureUsage: u8 {
        // This texture will be sampled within a shader
        const SAMPLED = 1;

        // This texture will be used as a render target attachment
        const RENDER_TARGET = 2 | Self::COPY_SRC.bits;

        // Data can be copied from the texture on the GPU side
        const COPY_SRC = 4;

        // Data can be copied into the texture on the GPU side
        // Required by the texture when we have pre-initialized data
        const COPY_DST = 8;

        // The texture can be used for reading GPU data back
        const READ = 16 | Self::COPY_SRC.bits;

        // The texture can be used to send data to the GPU
        const WRITE = 32 | Self::COPY_DST.bits;
    }
}

impl Default for TextureUsage {
    fn default() -> Self {
        Self::READ | Self::COPY_DST | Self::SAMPLED
    }
}
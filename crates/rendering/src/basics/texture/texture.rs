use gl::types::GLuint;

use super::{get_texel_byte_size, RawTexture, TextureBytes, TextureParams};

// Shared texture logic
pub trait Texture {
    // Dimension types
    type Dimensions: Default;

    // Get the raw underlying texture
    fn storage(&self) -> Option<&RawTexture>;
    // Get the texture target (OpenGL)
    fn target(&self) -> Option<GLuint> {
        self.storage().as_ref().map(|storage| storage.target)
    }
    // Get the underlying texture storage name
    fn name(&self) -> Option<GLuint> {
        self.storage().as_ref().map(|storage| storage.name)
    }
    // Get the texture parameters
    fn params(&self) -> &TextureParams;
    // Get the texture bytes
    fn bytes(&self) -> &TextureBytes;
    // Calculate the number of texels in the texture
    fn count_texels(&self) -> usize;
    // Calculate the number of bytes the texture *can* have
    fn count_bytes(&self) -> usize {
        self.count_texels() * get_texel_byte_size(self.params().layout.internal_format)
    }
    // Get the current texture dimensions
    fn dimensions(&self) -> Self::Dimensions;
}

// Resizable texture
pub trait ResizableTexture: Texture {
    // Resize the current texture
    fn resize(&mut self, dimensions: Self::Dimensions) -> Option<()> {
        self.resize_then_write(dimensions, Vec::new())?;
        Some(())
    }
    // Resize the current texture, then set it's bytes
    fn resize_then_write(&mut self, dimensions: Self::Dimensions, bytes: Vec<u8>) -> Option<()>;
}

// Writable texture
pub trait WritableTexture: Texture {
    // Set the contents of the texture
    fn write(&mut self, bytes: Vec<u8>) -> Option<()>;
    // Clear a texture
    fn clear(&mut self) {
        self.write(Vec::new()).unwrap()
    }
}

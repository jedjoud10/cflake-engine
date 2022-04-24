use crate::basics::texture::{Texture2D, Texture, get_texel_byte_size, TextureLayout, TextureFormat};

// Mask texture map builder that uses 3 textures to build a packed mask texture
#[derive(Default)]
pub struct MaskBuilder {
    textures: [Option<Texture2D>; 3],
    size: Option<vek::Extent2<u32>>
}

impl MaskBuilder {
    // Set a texture using an index
    pub fn set(&mut self, texture: Texture2D, i: usize) {
        let old = self.size.get_or_insert(texture.dimensions());
        assert_eq!(*old, texture.dimensions(), "Texture size mismatch");

        // Only works for RGBA4 textures
        assert_eq!(texture.params().layout.internal_format, TextureFormat::RGBA8R, "Mask builder only supports RGBA8 textures");

        self.textures[i] = Some(texture);
    }

    // Set the ambient occlusion texture
    pub fn ao(mut self, texture: Texture2D) -> Self {
        self.set(texture, 0);
        self
    }
    // Set the roughness texture
    pub fn roughness(mut self, texture: Texture2D) -> Self {
        self.set(texture, 1);
        self
    }
    // Set the metallic texture
    pub fn metallic(mut self, texture: Texture2D) -> Self {
        self.set(texture, 2);
        self
    }

    // Build the final mask texture
    pub fn build(self) -> Option<Texture2D> {
        // Funny fard moment?
        let size = self.size?;
        let mut bytes = vec![0u8; size.product() as usize * 4];
        let params = self
            .textures
            .iter()
            .find_map(|tex| tex.as_ref().map(|tex| tex.params()))?;

        // Iterate through the textures and update the bytes
        for (i, texture) in self.textures.iter().enumerate() {
            if let Some(texture) = texture {
                // Update the final bytes using the unique texture's bytes
                let byte_per_texel = get_texel_byte_size(texture.params().layout.internal_format);
                let mut old = texture.bytes().as_valid().unwrap().chunks(byte_per_texel);
                for bytes in bytes.chunks_mut(4) {
                    // Load a byte array from the old texture, then get the first value in the R channel
                    let single =  old.next().unwrap();
                    bytes[i] = single[0];
                }
            }
        }

        // Le return
        Some(Texture2D::new(size, Some(bytes), *params))
    }
}
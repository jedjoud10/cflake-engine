use std::{ffi::c_void, ptr::null};

use assets::Asset;

use crate::{object::Builder, pipeline::TextureKey, basics::texture::{TextureDimensions, get_ifd, calculate_size_bytes, TextureFormat, TextureFilter, TextureAccessType, TextureWrapping}, utils::DataType};

use super::Texture;

// Texture builder
pub struct TextureBuilder {
    inner: Texture
}

impl Default for TextureBuilder {

}

impl TextureBuilder {
    // The internal format and data type of the soon to be generated texture
    pub fn with_format(mut self, _format: TextureFormat) -> Self {
        self.inner._format = _format;
        self
    }
    // Set the data type for this texture
    pub fn with_data_type(mut self, _type: DataType) -> Self {
        self.inner._type = _type;
        self
    }
    // Set the height and width of the soon to be generated texture
    pub fn with_dimensions(mut self, ttype: TextureDimensions) -> Self {
        self.inner.dimensions = ttype;
        self
    }
    // Set the texture type
    pub fn with_type(mut self, ttype: TextureDimensions) -> Self {
        self.inner.dimensions = ttype;
        self
    }
    // Set the bytes of this texture
    pub fn with_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.inner.bytes = bytes;
        self
    }
    // We can read from this texture on the CPU, so we must create a Download PBO
    pub fn become_readable(mut self) -> Self {
        self.inner.cpu_access.insert(TextureAccessType::READ);
        self
    }
    // We can write to this texture on the CPU, so we must create an Upload PBO
    pub fn become_writable(mut self) -> Self {
        self.inner.cpu_access.insert(TextureAccessType::WRITE);
        self
    }
    // Set mipmaps
    pub fn with_mipmaps(mut self, enabled: bool) -> Self {
        self.inner.mipmaps = enabled;
        self
    }
    // Set the mag and min filters
    pub fn with_filter(mut self, filter: TextureFilter) -> Self {
        self.inner.filter = filter;
        self
    }
    // Set the wrapping mode
    pub fn with_wrapping_mode(mut self, wrapping_mode: TextureWrapping) -> Self {
        self.inner.wrap_mode = wrapping_mode;
        self
    }
    // Set the border colors
    pub fn with_border_colors(mut self, colors: [veclib::Vector4<f32>; 4]) -> Self {
        self.inner.border_colors = colors;
        self
    }
    // Set an OpenGL texture parameter for this texture
    pub fn with_custom_gl_param(mut self, name: u32, param: u32) -> Self {
        self.inner.custom_params.push((name, param));
        self
    }
}

impl Builder for TextureBuilder {
    type Key = TextureKey;
    type Element = Texture;

    fn build(self, slotmap: &mut slotmap::SlotMap<Self::Key, Self::Element>) -> Self::Key {
        let mut texture = self.inner;
        texture.ifd = get_ifd(texture._format, texture._type);
        texture.target = match texture.dimensions {
            TextureDimensions::Texture1D(_) => gl::TEXTURE_1D,
            TextureDimensions::Texture2D(_, _) => gl::TEXTURE_2D,
            TextureDimensions::Texture3D(_, _, _) => gl::TEXTURE_3D,
            TextureDimensions::Texture2DArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
        };
        // Guess how many mipmap levels a texture with a specific maximum coordinate can have
        fn guess_mipmap_levels(i: usize) -> usize {
            let mut x: f32 = i as f32;
            let mut num: usize = 0;
            while x > 1.0 {
                // Repeatedly divide by 2
                x /= 2.0;
                num += 1;
            }
            num
        }

        let pointer: *const c_void = if !texture.bytes.is_empty() { texture.bytes.as_ptr() as *const c_void } else { null() };

        let ifd = get_ifd(texture._format, texture._type);
        let bytes_count = calculate_size_bytes(&texture._format, texture.count_pixels());

        // Get the tex_type based on the TextureDimensionType
        let tex_type = match self.dimensions {
            TextureDimensions::Texture1D(_) => gl::TEXTURE_1D,
            TextureDimensions::Texture2D(_, _) => gl::TEXTURE_2D,
            TextureDimensions::Texture3D(_, _, _) => gl::TEXTURE_3D,
            TextureDimensions::Texture2DArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
        };
        let texel_count = self.count_pixels();

        let mut oid: u32 = 0;
        unsafe {
            gl::GenTextures(1, &mut oid as *mut u32);
            gl::BindTexture(tex_type, oid);
            if texel_count > 0 {
                match self.dimensions {
                    TextureDimensions::Texture1D(width) => {
                        gl::TexImage1D(tex_type, 0, ifd.0, width as i32, 0, ifd.1, ifd.2, pointer);
                    }
                    // This is a 2D texture
                    TextureDimensions::Texture2D(width, height) => {
                        gl::TexImage2D(tex_type, 0, ifd.0, width as i32, height as i32, 0, ifd.1, ifd.2, pointer);
                    }
                    // This is a 3D texture
                    TextureDimensions::Texture3D(width, height, depth) => {
                        gl::TexImage3D(tex_type, 0, ifd.0, width as i32, height as i32, depth as i32, 0, ifd.1, ifd.2, pointer);
                    }
                    // This is a texture array
                    TextureDimensions::Texture2DArray(width, height, depth) => {
                        gl::TexStorage3D(
                            tex_type,
                            guess_mipmap_levels(width.max(height) as usize) as i32,
                            ifd.0 as u32,
                            width as i32,
                            height as i32,
                            depth as i32,
                        );
                        // We might want to do mipmap
                        for i in 0..depth {
                            let localized_bytes = self.bytes[(i as usize * height as usize * 4 * width as usize)..self.bytes.len()].as_ptr() as *const c_void;
                            gl::TexSubImage3D(gl::TEXTURE_2D_ARRAY, 0, 0, 0, i as i32, width as i32, height as i32, 1, ifd.1, ifd.2, localized_bytes);
                        }
                    }
                }
            }
            // Set the texture parameters for a normal texture
            match self.filter {
                TextureFilter::Linear => {
                    // 'Linear' filter
                    gl::TexParameteri(tex_type, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                    gl::TexParameteri(tex_type, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                }
                TextureFilter::Nearest => {
                    // 'Nearest' filter
                    gl::TexParameteri(tex_type, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                    gl::TexParameteri(tex_type, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                }
            }
        }

        // The texture is already bound to the TEXTURE_2D
        if self.mipmaps {
            // Create the mipmaps
            unsafe {
                gl::GenerateMipmap(tex_type);
                // Set the texture parameters for a mipmapped texture
                match self.filter {
                    TextureFilter::Linear => {
                        // 'Linear' filter
                        gl::TexParameteri(tex_type, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);
                        gl::TexParameteri(tex_type, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                    }
                    TextureFilter::Nearest => {
                        // 'Nearest' filter
                        gl::TexParameteri(tex_type, gl::TEXTURE_MIN_FILTER, gl::NEAREST_MIPMAP_NEAREST as i32);
                        gl::TexParameteri(tex_type, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                    }
                }
            }
        }

        // Create the Upload / Download PBOs if needed
        if self.cpu_access.contains(TextureAccessType::READ) {
            // Create a download PBO
            let mut pbo = 0_u32;
            unsafe {
                gl::GenBuffers(1, &mut pbo);
                gl::BindBuffer(gl::PIXEL_PACK_BUFFER, pbo);
                gl::BufferData(gl::PIXEL_PACK_BUFFER, bytes_count as isize, null(), gl::STREAM_COPY);
                gl::BindBuffer(gl::PIXEL_PACK_BUFFER, 0);
            }
            self.read_pbo = Some(pbo);
        } else if self.cpu_access.contains(TextureAccessType::WRITE) {
            // Create an upload PBO
            let mut pbo = 0_u32;
            unsafe {
                gl::GenBuffers(1, &mut pbo);
                gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, pbo);
                gl::BufferData(gl::PIXEL_UNPACK_BUFFER, bytes_count as isize, null(), gl::STREAM_DRAW);
                gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, 0);
            }
            self.write_pbo = Some(pbo);
        }

        // Set the wrap mode for the texture (Mipmapped or not)
        let wrapping_mode = match self.wrap_mode {
            TextureWrapping::ClampToEdge => gl::CLAMP_TO_EDGE,
            TextureWrapping::ClampToBorder => gl::CLAMP_TO_BORDER,
            TextureWrapping::Repeat => gl::REPEAT,
            TextureWrapping::MirroredRepeat => gl::MIRRORED_REPEAT,
        };
        unsafe {
            // Now set the actual wrapping mode in the opengl texture
            gl::TexParameteri(tex_type, gl::TEXTURE_WRAP_S, wrapping_mode as i32);
            gl::TexParameteri(tex_type, gl::TEXTURE_WRAP_T, wrapping_mode as i32);
            // And also border colors
            use veclib::Vector;
            let ptr = texture.border_colors.get(0).unwrap().as_ptr();
            gl::TexParameterfv(tex_type, gl::TEXTURE_BORDER_COLOR, ptr);
        }

        // Set the custom parameter
        for (name, param) in &texture.custom_params {
            unsafe {
                gl::TexParameteri(tex_type, *name, *param as i32);
            }
        }

        // Add the texture
        texture.oid = oid;
        unsafe {
            gl::BindTexture(tex_type, 0);
        }
        slotmap.insert(texture)
    }
}

impl Asset for TextureBuilder {
    fn load_raw(meta: &assets::metadata::AssetMetadata, bytes: &[u8]) -> Option<Self> {
        // Read bytes
        pub fn read_bytes(bytes: &[u8]) -> (Vec<u8>, u16, u16) {
            // Load this texture from the bytes
            let image = image::load_from_memory(bytes).unwrap();
            let image = image::DynamicImage::ImageRgba8(image.into_rgba8());
            // Flip
            let image = image.flipv();
            (image.to_bytes(), image.width() as u16, image.height() as u16)
        }
        // Load this texture from the bytes
        let (bytes, width, height) = read_bytes(bytes);

        // Return a texture with the default parameters
        let builder = Self::default()
            .with_bytes(bytes)
            .with_dimensions(TextureDimensions::Texture2D(width, height))
            .with_format(TextureFormat::RGBA8R)
            .with_data_type(DataType::U8);
        Some(builder)
    }
}
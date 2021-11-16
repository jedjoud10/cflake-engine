use assets::*;
use gl;
use image::{DynamicImage, EncodableLayout, GenericImageView};
use std::{ffi::c_void, ptr::null, rc::Rc};
use rendering_base::{error::RenderingError, main_types::DataType};
use rendering_base::texture::*;
use crate::{RenderAsset, RenderObject, TextureT};

// Create the type alias
pub type Texture = rendering_base::texture::Texture;

// Loadable asset
impl RenderAsset for Texture {
    // Load a texture from scratch
    fn asset_load(data: &AssetMetadata) -> Option<Self>
    where
        Self: Sized,
    {
        // Load this texture from the bytes
        let (bytes, width, height) = Self::read_bytes(data);
        // Return a texture with the default parameters
        let texture = Texture::new()
            .set_dimensions(TextureType::Texture2D(width, height))
            .set_format(TextureFormat::RGBA8R)
            .set_data_type(DataType::UByte)
            .set_name(&data.name)
            .generate_texture(bytes)
            .unwrap();
        return Some(texture);
    }
    // Load a texture that already has it's parameters set
    fn asset_load_t(self, data: &AssetMetadata) -> Option<Self>
    where
        Self: Sized,
    {
        let (bytes, width, height) = Self::read_bytes(data);
        let texture = self.set_name(&data.name).set_dimensions(TextureType::Texture2D(width, height));
        // Return a texture with the default parameters
        let texture = texture.generate_texture(bytes).unwrap();
        return Some(texture);
    }
}
// Render object
impl RenderObject for Texture {
    fn new() -> Self {
        Self::default()
    }
    fn finalize(self) -> Self where Self: Sized {
        self
    }    
    
}

// Get the IDF from a simple TextureFormat and DataType
fn get_idf(tf: TextureFormat, dt: DataType) -> (i32, u32, u32) {
    let internal_format = match tf {
        TextureFormat::R8R => gl::R8,
        TextureFormat::R8I => gl::R8I,
        TextureFormat::R16I => gl::R16I,
        TextureFormat::R32I => gl::R32I,
        TextureFormat::R16F => gl::R16F,
        TextureFormat::R32f => gl::R32F,
        TextureFormat::RG8R => gl::RG8,
        TextureFormat::RG8I => gl::RG8I,
        TextureFormat::RG16I => gl::RG16I,
        TextureFormat::RG32I => gl::RG32I,
        TextureFormat::RG16F => gl::RG16F,
        TextureFormat::RG32F => gl::RG32F,
        TextureFormat::RGB8R => gl::RGB8,
        TextureFormat::RGB8I => gl::RGB8I,
        TextureFormat::RGB16I => gl::RGB16I,
        TextureFormat::RGB32I => gl::RGB32I,
        TextureFormat::RGB16F => gl::RGB16F,
        TextureFormat::RGB32F => gl::RGB32F,
        TextureFormat::RGBA8R => gl::RGBA8,
        TextureFormat::RGBA8I => gl::RGBA8I,
        TextureFormat::RGBA16I => gl::RGBA16I,
        TextureFormat::RGBA32I => gl::RGBA32I,
        TextureFormat::RGBA16F => gl::RGBA16F,
        TextureFormat::RGBA32F => gl::RGBA32F,
    };
    // Get the format of this texture using it's name
    let name = format!("{:?}", tf);
    panic!(name);
    let format = 0;
    let data_type = 0;
    (internal_format as i32, format, data_type)
}

impl TextureT for Texture {    
    // Read bytes
    fn read_bytes(metadata: &AssetMetadata) -> (Vec<u8>, u16, u16) {
        // Load this texture from the bytes
        let png_bytes = metadata.bytes.as_bytes();
        let image = image::load_from_memory_with_format(png_bytes, image::ImageFormat::Png).unwrap();
        // Flip
        let image = image.flipv();
        return (image.to_bytes(), image.width() as u16, image.height() as u16);
    }
    // Update the size of the current texture
    fn update_size(&mut self, ttype: TextureType) {
        // Check if the current dimension type matches up with the new one
        self.ttype = ttype;
        let (internal_format, format, data_type) = get_idf(self._format, self._type);
        // This is a normal texture getting resized
        unsafe {
            match self.ttype {
                TextureType::Texture1D(width) => {
                    gl::BindTexture(gl::TEXTURE_1D, self.id);
                    gl::TexImage1D(gl::TEXTURE_2D, 0, internal_format, width as i32, 0, format, data_type, null());
                }
                TextureType::Texture2D(width, height) => {
                    gl::BindTexture(gl::TEXTURE_2D, self.id);
                    gl::TexImage2D(
                        gl::TEXTURE_2D,
                        0,
                        internal_format,
                        width as i32,
                        height as i32,
                        0,
                        format,
                        data_type,
                        null(),
                    );
                }
                TextureType::Texture3D(width, height, depth) => {
                    gl::BindTexture(gl::TEXTURE_3D, self.id);
                    gl::TexImage3D(
                        gl::TEXTURE_3D,
                        0,
                        internal_format,
                        width as i32,
                        height as i32,
                        depth as i32,
                        0,
                        format,
                        data_type,
                        null(),
                    );
                }
                TextureType::TextureArray(_, _, _) => todo!(),
            }
        }
    }  
    // Create a texture array from multiple texture paths (They must have the same dimensions!)
    fn create_texturearray(load_options: Option<TextureLoadOptions>, texture_paths: Vec<&str>, asset_manager: &mut AssetManager, width: u16, height: u16) -> Rc<Texture> {
        // Load the textures
        let mut bytes: Vec<u8> = Vec::new();
        let name = &format!("{}-{}", "2dtexturearray", texture_paths.join("--"));
        let length = texture_paths.len();
        for x in texture_paths {
            // Load this texture from the bytes
            let metadata = asset_manager.asset_cacher.load_md(x).unwrap();
            let png_bytes = metadata.bytes.as_bytes();
            let image = image::load_from_memory_with_format(png_bytes, image::ImageFormat::Png).unwrap();
            // Resize the image so it fits the dimension criteria
            let image = image.resize_exact(width as u32, height as u32, image::imageops::FilterType::Gaussian);
            // Flip
            let image = image.flipv();
            let bytesa = image.to_bytes();
            bytes.extend(bytesa);
        }
        // Create the array texture from THOSE NUTS AAAAA
        let main_texture: Texture = Texture::new()
            .enable_mipmaps()
            .set_dimensions(TextureType::TextureArray(width, height, length as u16))
            .set_format(TextureFormat::RGBA8R)
            .set_name(name)
            .apply_texture_load_options(load_options)
            .generate_texture(bytes)
            .unwrap();
        main_texture.object_cache_load(name, &mut asset_manager.object_cacher)
    }
    // Generate an empty texture, could either be a mutable one or an immutable one
    fn generate_texture(self, bytes: Vec<u8>) -> Result<Self, RenderingError> {
        let mut texture = self; 
        let mut pointer: *const c_void = null();
        if !bytes.is_empty() {
            pointer = bytes.as_ptr() as *const c_void;
        }

        // Get the tex_type based on the TextureDimensionType
        let (internal_format, format, data_type) = get_idf(self._format, self._type);
        let tex_type = match self.ttype {
            TextureType::Texture1D(_) => gl::TEXTURE_1D,
            TextureType::Texture2D(_, _) => gl::TEXTURE_2D,
            TextureType::Texture3D(_, _, _) => gl::TEXTURE_3D,
            TextureType::TextureArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
        };

        // It's a normal mutable texture
        unsafe {
            gl::GenTextures(1, &mut self.id as *mut u32);
            gl::BindTexture(tex_type, self.id);
            match self.ttype {
                TextureType::Texture1D(_) => {
                    gl::TexImage1D(tex_type, 0, internal_format, self.get_width() as i32, 0, format, data_type, pointer);
                }
                // This is a 2D texture
                TextureType::Texture2D(_, _) => {
                    gl::TexImage2D(
                        tex_type,
                        0,
                        internal_format,
                        self.get_width() as i32,
                        self.get_height() as i32,
                        0,
                        format,
                        data_type,
                        pointer,
                    );
                }
                // This is a 3D texture
                TextureType::Texture3D(_, _, _) => {
                    gl::TexImage3D(
                        tex_type,
                        0,
                        internal_format,
                        self.get_width() as i32,
                        self.get_height() as i32,
                        self.get_depth() as i32,
                        0,
                        format,
                        data_type,
                        pointer,
                    );
                }
                // This is a texture array
                TextureType::TextureArray(x, y, l) => {
                    gl::TexStorage3D(
                        tex_type,
                        Self::guess_mipmap_levels(x.max(y) as usize) as i32,
                        internal_format as u32,
                        x as i32,
                        y as i32,
                        l as i32,
                    );
                    // We might want to do mipmap
                    for i in 0..l {
                        let localized_bytes = bytes[(i as usize * y as usize * 4 * x as usize)..bytes.len()].as_ptr() as *const c_void;
                        gl::TexSubImage3D(
                            gl::TEXTURE_2D_ARRAY,
                            0,
                            0,
                            0,
                            i as i32,
                            self.get_width() as i32,
                            self.get_height() as i32,
                            1,
                            format,
                            data_type,
                            localized_bytes,
                        );
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
        if self.flags.contains(TextureFlags::MIPMAPS) {
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
        

        // Set the wrap mode for the texture (Mipmapped or not)
        let wrapping_mode: i32;
        match self.wrap_mode {
            TextureWrapping::ClampToEdge => wrapping_mode = gl::CLAMP_TO_EDGE as i32,
            TextureWrapping::ClampToBorder => wrapping_mode = gl::CLAMP_TO_BORDER as i32,
            TextureWrapping::Repeat => wrapping_mode = gl::REPEAT as i32,
            TextureWrapping::MirroredRepeat => wrapping_mode = gl::MIRRORED_REPEAT as i32,
        }
        unsafe {
            // Now set the actual wrapping mode in the opengl texture
            gl::TexParameteri(tex_type, gl::TEXTURE_WRAP_S, wrapping_mode);
            gl::TexParameteri(tex_type, gl::TEXTURE_WRAP_T, wrapping_mode);
        }
        println!("Succsesfully generated texture {}", self.name);
        Ok(self)
    }
    // Update a valid texture's data
    fn update_data(&mut self, bytes: Vec<u8>) {
        let mut pointer: *const c_void = null();
        if !bytes.is_empty() {
            pointer = bytes.as_ptr() as *const c_void;
        }

        let (internal_format, format, data_type) = get_idf(self._format, self._type);
        let tex_type = match self.ttype {
            TextureType::Texture1D(_) => gl::TEXTURE_1D,
            TextureType::Texture2D(_, _) => gl::TEXTURE_2D,
            TextureType::Texture3D(_, _, _) => gl::TEXTURE_3D,
            TextureType::TextureArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
        };

        unsafe {            
            gl::BindTexture(tex_type, self.id);
            match self.ttype {
                TextureType::Texture1D(_) => gl::TexImage1D(tex_type, 0, internal_format, self.get_width() as i32, 0, format, data_type, pointer),
                // This is a 2D texture
                TextureType::Texture2D(_, _) => {
                    gl::TexImage2D(
                        tex_type,
                        0,
                        internal_format,
                        self.get_width() as i32,
                        self.get_height() as i32,
                        0,
                        format,
                        data_type,
                        pointer,
                    );
                }
                // This is a 3D texture
                TextureType::Texture3D(_, _, _) => {
                    gl::TexImage3D(
                        tex_type,
                        0,
                        internal_format,
                        self.get_width() as i32,
                        self.get_height() as i32,
                        self.get_depth() as i32,
                        0,
                        format,
                        data_type,
                        pointer,
                    );
                }
                // This is a texture array
                TextureType::TextureArray(x, y, l) => {
                    gl::TexStorage3D(tex_type, 10, internal_format as u32, x as i32, y as i32, l as i32);
                    // We might want to do mipmap
                    for i in 0..l {
                        let localized_bytes = bytes[(i as usize * y as usize * 4 * x as usize)..bytes.len()].as_ptr() as *const c_void;
                        gl::TexSubImage3D(
                            gl::TEXTURE_2D_ARRAY,
                            0,
                            0,
                            0,
                            i as i32,
                            self.get_width() as i32,
                            self.get_height() as i32,
                            1,
                            format,
                            data_type,
                            localized_bytes,
                        );
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
    }
    /*
    // Get the image from this texture and fill an array of vec2s, vec3s or vec4s with it
    pub fn fill_array_veclib<V, U>(&self) -> Vec<V>
    where
        V: veclib::Vector<U> + Default + Clone,
        U: veclib::DefaultStates,
    {
        // Get the length of the vector
        let length: usize = match self.ttype {
            TextureType::Texture1D(x) => (x as usize),
            TextureType::Texture2D(x, y) => (x as usize * y as usize),
            TextureType::Texture3D(x, y, z) => (x as usize * y as usize * z as usize),
            TextureType::TextureArray(_, _, _) => todo!(),
        };
        // Create the vector
        let mut pixels: Vec<V> = vec![V::default(); length];

        let tex_type = match self.ttype {
            TextureType::Texture1D(_) => gl::TEXTURE_1D,
            TextureType::Texture2D(_, _) => gl::TEXTURE_2D,
            TextureType::Texture3D(_, _, _) => gl::TEXTURE_3D,
            TextureType::TextureArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
        };

        // Actually read the pixels
        unsafe {
            // Bind the buffer before reading
            gl::BindTexture(tex_type, self.id);
            gl::GetTexImage(tex_type, 0, self.format, self.data_type, pixels.as_mut_ptr() as *mut c_void);
        }
        return pixels;
    }
    // Get the image from this texture and fill an array of single elements with it
    pub fn fill_array_elems<U>(&self) -> Vec<U>
    where
        U: Clone + Default,
    {
        // Get the length of the vector
        let length: usize = match self.ttype {
            TextureType::Texture1D(x) => (x as usize),
            TextureType::Texture2D(x, y) => (x as usize * y as usize),
            TextureType::Texture3D(x, y, z) => (x as usize * y as usize * z as usize),
            TextureType::TextureArray(_, _, _) => todo!(),
        };
        // Create the vector
        let mut pixels: Vec<U> = vec![U::default(); length];

        let tex_type = match self.ttype {
            TextureType::Texture1D(_) => gl::TEXTURE_1D,
            TextureType::Texture2D(_, _) => gl::TEXTURE_2D,
            TextureType::Texture3D(_, _, _) => gl::TEXTURE_3D,
            TextureType::TextureArray(_, _, _) => gl::TEXTURE_2D_ARRAY,
        };

        // Actually read the pixels
        unsafe {
            // Bind the buffer before reading
            gl::BindTexture(tex_type, self.id);
            gl::GetTexImage(tex_type, 0, self.format, self.data_type, pixels.as_mut_ptr() as *mut c_void);
        }
        return pixels;
    }
    */
}

use std::{ops::{Index, IndexMut}, ffi::c_void};

use crate::{basics::texture::{RawTexture, TextureBytes, TextureParams, Texture, TextureLayout, generate_mipmaps, generate_filters, TextureFlags, guess_mipmap_levels}, utils::DataType, object::ObjectSealed};

use super::Texture2D;

// A cubemap that contains 6 different textures, combined into a unit cube
pub struct CubeMap {
    // Storage
    raw: Option<RawTexture>,
    
    // The texture bytes
    bytes: TextureBytes,
    
    // Params
    params: TextureParams,
    
    // A face's dimensions
    dimensions: vek::Extent2<u32>,
}

// Cubemap textures used for initialization
pub struct CubeMapTextures {
    // Six textures indeed
    /*
    GL_TEXTURE_CUBE_MAP_POSITIVE_X	Right
    GL_TEXTURE_CUBE_MAP_NEGATIVE_X	Left
    GL_TEXTURE_CUBE_MAP_POSITIVE_Y	Top
    GL_TEXTURE_CUBE_MAP_NEGATIVE_Y	Bottom
    GL_TEXTURE_CUBE_MAP_POSITIVE_Z	Back
    GL_TEXTURE_CUBE_MAP_NEGATIVE_Z	Front
    */
    inner: [Texture2D; 6],
}

impl Index<usize> for CubeMapTextures {
    type Output = Texture2D;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl IndexMut<usize> for CubeMapTextures {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

impl CubeMapTextures {
    // Create some cubemap textures using texture directions
    pub fn new(right: Texture2D, left: Texture2D, top: Texture2D, bottom: Texture2D, back: Texture2D, front: Texture2D) -> Self {
        Self {
            inner: [right, left, top, bottom, back, front]
        }
    }
}


impl CubeMap {
    // Create a cubemap using 6 different textures and some initialization parameters
    // PS: This will return None if one of the textures has a different size than the others
    pub fn new(textures: CubeMapTextures, params: TextureParams) -> Option<Self> {
        // Doesn't matter which is the "main" texture
        let dimensions = textures[0].dimensions();
        
        // Combined bytes that will be allocated to the cubemap
        let mut bytes = Vec::<u8>::with_capacity(dimensions.product() as usize * 4 * 6);

        // Valid layout for textures
        // For now, cubemaps only support RGBA8 textures
        const LAYOUT: TextureLayout = TextureLayout::LOADED;
        if params.layout != LAYOUT { return None; }
        
        for texture in textures.inner {
            // Make sure the dimensions match up, and return if it does not
            // Also make sure that the layout is valid
            if texture.dimensions() != dimensions || texture.params().layout != LAYOUT {
                return None;
            }

            // Add the texture bytes to the main bytes
            let texture_bytes = texture.bytes().as_valid().unwrap();
            bytes.extend_from_slice(&texture_bytes);
        }

        // Construct the cubemap using the valid bytes
        Some(Self {
            raw: None,
            bytes: TextureBytes::Valid(bytes),
            params,
            dimensions,
        })
    }
}

impl Texture for CubeMap {
    type Dimensions = vek::Extent2<u32>;

    fn storage(&self) -> Option<&RawTexture> {
        self.raw.as_ref()
    }

    fn params(&self) -> &TextureParams {
        &self.params
    }

    fn bytes(&self) -> &TextureBytes {
        &self.bytes
    }

    fn count_texels(&self) -> usize {
        self.dimensions().as_::<usize>().product() * 6
    }

    fn dimensions(&self) -> Self::Dimensions {
        self.dimensions
    }
}

impl ObjectSealed for CubeMap {
    fn init(&mut self, _pipeline: &mut crate::pipeline::Pipeline) {
        // TODO: Fix code duplication between bundledtexture2d and texture2d
        // Create the raw texture wrapper
        let texture = unsafe { RawTexture::new(gl::TEXTURE_CUBE_MAP, &self.params) };
        let ifd = texture.ifd;
        self.raw = Some(texture);

        // Number of bytes per cubemap face
        let per_face = self.dimensions.product() as usize * 4;
        let (width, height) = self.dimensions.as_::<i32>().into_tuple();

        // Texture generation, SRGB, mipmap, filters
        // Cubemap textures are not resizable, for now
        unsafe {
            // Don't allocate anything if the textures dimensions are invalid
            if per_face != 0 {
                let base_ptr = self.bytes.get_ptr() as *const c_void;
                let levels = guess_mipmap_levels(self.dimensions.reduce_max()).max(1) as i32;
                

                /*
                
                    int width, height, nrChannels;
                    unsigned char *data;  
                    for(unsigned int i = 0; i < textures_faces.size(); i++)
                    {
                        data = stbi_load(textures_faces[i].c_str(), &width, &height, &nrChannels, 0);
                        glTexImage2D(
                            GL_TEXTURE_CUBE_MAP_POSITIVE_X + i, 
                            0, GL_RGB, width, height, 0, GL_RGB, GL_UNSIGNED_BYTE, data
                        );
                    }
                */


                for face in 0..6 {
                    // Get the corresponding byte range for this face
                    let start = face * per_face;
                    let end = (face+1) * per_face;

                    // And get the corresponding pointer
                    let ptr = base_ptr.add(start);

                    // Le procedural target?
                    let target = gl::TEXTURE_CUBE_MAP_POSITIVE_X + face as u32;
                    
                    gl::TexStorage2D(target, levels, ifd.0, width, height);
                    if !ptr.is_null() {
                        gl::TexSubImage2D(target, 0, 0, 0, width, height, ifd.1, ifd.2, ptr);
                    }
                }
            }

            // Mipmaps and filters
            generate_mipmaps(gl::TEXTURE_CUBE_MAP, &self.params);
            generate_filters(gl::TEXTURE_CUBE_MAP, &self.params);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32); 
        }

        // Clear the texture if it's loaded bytes aren't persistent
        if !self.params.flags.contains(TextureFlags::PERSISTENT) {
            self.bytes.clear();
        }
    }
}
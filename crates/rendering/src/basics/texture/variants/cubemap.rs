use std::{ops::{Index, IndexMut}, ffi::c_void, mem::MaybeUninit};
use arrayvec::ArrayVec;
use assets::Asset;
use gl::types::GLuint;
use crate::{basics::{texture::{RawTexture, TextureBytes, TextureParams, Texture, TextureLayout, generate_mipmaps, generate_filters, TextureFlags, guess_mipmap_levels, TextureWrapMode, TextureFilter, get_ifd}, shader::{Shader, ShaderInitSettings}}, utils::DataType, object::ObjectSealed, pipeline::{Pipeline, Handle, Framebuffer, FramebufferClearBits}};
use super::Texture2D;

// A cubemap face texture that simply contains an index of it's face
// This will be merged with 6 more face textures to construct a single cube map
struct FaceTexture2D {
    // Storage
    raw: Option<RawTexture>,

    // The texture bytes
    bytes: TextureBytes,

    // Texture dimensions (always square)
    size: u32,

    // Face index
    index: u32
}

impl FaceTexture2D {
    // Parameters for a face texture
    const PARAMS: TextureParams = TextureParams {
        layout: TextureLayout::HDR,
        filter: TextureFilter::Linear,
        wrap: TextureWrapMode::ClampToEdge,
        flags: TextureFlags::empty(),
    };

    // Constant IFD since we know the parameters
    const IFD: (GLuint, GLuint, GLuint) = get_ifd(Self::PARAMS.layout);
}

impl ObjectSealed for FaceTexture2D {
    fn init(&mut self, _pipeline: &mut crate::pipeline::Pipeline) {
        // Custom target for each face
        let target = gl::TEXTURE_CUBE_MAP_POSITIVE_X + self.index;

        // Create the raw texture
        self.raw = Some( unsafe { RawTexture::new(target, &Self::PARAMS) });
        let size = self.size as i32;
        let ifd = Self::IFD;

        // We can assume a lot since we know what is a cubemap exactly
        unsafe {
            // The pointer always points to valid data
            let ptr = self.bytes.get_ptr() as *const c_void;

            // Face textures are always static, they can never resize
            gl::TexStorage2D(target, 1, ifd.0, size, size);
            gl::TexSubImage2D(target, 0, 0, 0, size, size, ifd.1, ifd.2, ptr);
        }

        // Face textures never keep their bytes, since the main cubemap will store that (if needed)
        self.bytes.clear();
    }
}




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

// Generate 6 cubemap face textures with a specified side length
fn generate_textures(pipeline: &mut Pipeline, size: u32) -> [Handle<FaceTexture2D>; 6] {
    // Allocate 6 textures for the projection
    let mut textures = ArrayVec::<Handle<FaceTexture2D>, 6>::new();
    for i in 0..6 {
        // Create an empty texture, since we will be writing to it afterwards
        let texture = pipeline.insert(FaceTexture2D {
            raw: None,
            bytes: TextureBytes::Valid(Vec::new()),
            size,
            index: i,
        });

        // Add the texture internally (TODO: Remove the for loop and simplify)
        textures.push(texture);
    }
    textures.into_inner().unwrap()
}

impl CubeMap {
    // Create a cubemap using a single equirectangular map, like an HDR
    // This will project the map onto a sphere, and then render a unit cube 6 times for each face of the cubemap
    pub fn from_equirectangular(pipeline: &mut Pipeline, texture: Texture2D, params: TextureParams, size: u32) -> Option<Handle<Self>> {
        // Make sure the texture's layout can be used for an HDR cubemap
        if texture.params().layout != TextureParams::NON_COLOR_MAP_LOAD.layout {
            return None;
        }

        /*
            GL_TEXTURE_CUBE_MAP_POSITIVE_X	Right
            GL_TEXTURE_CUBE_MAP_NEGATIVE_X	Left
            GL_TEXTURE_CUBE_MAP_POSITIVE_Y	Top
            GL_TEXTURE_CUBE_MAP_NEGATIVE_Y	Bottom
            GL_TEXTURE_CUBE_MAP_POSITIVE_Z	Back
            GL_TEXTURE_CUBE_MAP_NEGATIVE_Z	Front
        */

        // Create the perspective matrix, and the 6 view matrices
        let perspective = vek::Mat4::perspective_fov_rh_no(90.0f32.to_radians(), 1.0, 1.0, 0.2, 2.0);
        use vek::Mat4;
        use vek::Vec3;

        // View matrices for the 6 different faces 
        let view_matrices: [Mat4<f32>; 6] = [
            Mat4::look_at_rh(Vec3::zero(), Vec3::unit_x(), -Vec3::unit_y()), // Right
            Mat4::look_at_rh(Vec3::zero(), -Vec3::unit_x(), -Vec3::unit_y()), // Left

            Mat4::look_at_rh(Vec3::zero(), Vec3::unit_y(), Vec3::unit_z()), // Top
            Mat4::look_at_rh(Vec3::zero(), -Vec3::unit_y(), -Vec3::unit_z()), // Bottom

            Mat4::look_at_rh(Vec3::zero(), Vec3::unit_z(), -Vec3::unit_y()), // Back
            Mat4::look_at_rh(Vec3::zero(), -Vec3::unit_z(), -Vec3::unit_y()), // Front
        ];
        
        // Create 6 cubemap face textures
        let textures = generate_textures(pipeline, size);

        // Load the shader that we will use for projection
        let shader = Shader::new(ShaderInitSettings::default()
            .source("defaults/shaders/rendering/project.vrsh.glsl")
            .source("defaults/shaders/others/cubemap.frsh.glsl")).unwrap();
        let shader = pipeline.insert(shader);
        
        // Create a framebuffer that will be used for rendering
        let mut framebuffer = Framebuffer::new(pipeline);
        
        // Render the cube 6 times with the appropriate shader and render target
        framebuffer.bind(|mut bound| {
            for (view, texture) in view_matrices.into_iter().zip(textures.into_iter()) {
                // Each time we render, we change the target texture
                //bound.target();
                bound.clear(FramebufferClearBits::all());
            }                
        });


        None
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

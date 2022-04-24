use std::{ops::{Index, IndexMut}, ffi::c_void, mem::MaybeUninit, ptr::null};
use arrayvec::ArrayVec;
use assets::Asset;
use gl::types::GLuint;
use crate::{basics::{texture::{RawTexture, TextureBytes, TextureParams, Texture, TextureLayout, generate_mipmaps, generate_filters, TextureFlags, guess_mipmap_levels, TextureWrapMode, TextureFilter, get_ifd}, shader::{Shader, ShaderInitSettings}, uniforms::Uniforms, mesh::{Mesh, GeometryBuilder}}, utils::DataType, object::ObjectSealed, pipeline::{Pipeline, Handle, Framebuffer, FramebufferClearBits, render}};
use super::Texture2D;

// How we will create the cubemap
enum CubeMapSource {
    HDR(Handle<Texture2D>),
}

// A cubemap that contains 6 different textures, combined into a unit cube
pub struct CubeMap {
    // Storage
    raw: Option<RawTexture>,
    
    // Texture dimensions for one face (the faces are always square)
    size: u32,

    // How we will create the cube map
    source: CubeMapSource,
}

impl CubeMap {
    // Parameters for the whole cubemap
    const PARAMS: TextureParams = TextureParams {
        layout: TextureLayout::HDR,
        filter: TextureFilter::Linear,
        wrap: TextureWrapMode::ClampToEdge,
        flags: TextureFlags::MIPMAPS,
    };

    // Constant IFD since we know the parameters
    const IFD: (GLuint, GLuint, GLuint) = get_ifd(Self::PARAMS.layout);
     
}

impl CubeMap {
    // Create a cubemap using a single equirectangular map, like an HDR
    // This will project the map onto a sphere, and then render a unit cube 6 times for each face of the cubemap
    pub fn from_equirectangular(hdr: Handle<Texture2D>, size: u32) -> Self {
        Self {
            raw: None,
            size,
            source: CubeMapSource::HDR(hdr),
        } 
    }
}

impl Texture for CubeMap {
    type Dimensions = u32;

    fn storage(&self) -> Option<&RawTexture> {
        self.raw.as_ref()
    }

    fn params(&self) -> &TextureParams {
        &Self::PARAMS
    }

    fn bytes(&self) -> &TextureBytes {
        &TextureBytes::Invalid
    }

    fn count_texels(&self) -> usize {
        (self.size as usize).pow(2) * 6
    }

    fn dimensions(&self) -> Self::Dimensions {
        self.size
    }
}

impl ObjectSealed for CubeMap {
    fn init(&mut self, pipeline: &mut crate::pipeline::Pipeline) {
        // TODO: Fix code duplication between bundledtexture2d and texture2d
        // Create the raw texture wrapper
        let texture = unsafe { RawTexture::new(gl::TEXTURE_CUBE_MAP, &Self::PARAMS) };
        let ifd = Self::IFD;
        let size = self.size;
        self.raw = Some(texture);
        unsafe {
            // Create the cubemap using the given source (currently only suports HDR captures)
            match &self.source {
                CubeMapSource::HDR(hdr) => {
                    // Make sure the texture's layout can be used for an HDR cubemap
                    assert_eq!(pipeline.get(&hdr).unwrap().params().layout, TextureParams::HDR_MAP_LOAD.layout, "HDR Texture layout invalid");
                
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

                    // Cubemaps always have mipmaps
                    let levels = guess_mipmap_levels(size).max(1) as i32;

                    // Allocate 6 textures for the projection
                    for i in 0..6 {
                        // Initialize a single face in the cubemap
                        gl::TexStorage2D(gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32, levels, ifd.0, size as i32, size as i32);
                    }
                    
                    // Load the shader that we will use for projection
                    // TODO: FIX OBJECT DUPLICATIONNN
                    let shader = pipeline.insert(Shader::new(ShaderInitSettings::default()
                        .source("defaults/shaders/rendering/project.vrsh.glsl")
                        .source("defaults/shaders/others/cubemap.frsh.glsl")).unwrap());

                    // Unit cube that is inside out
                    let cube = assets::load::<Mesh>("defaults/meshes/cube.obj").unwrap();
                    let cube = pipeline.insert(cube.flip_triangles());

                    // Fetch the added objects
                    let shader = pipeline.get(&shader).unwrap();
                    let mesh = pipeline.get(&cube).unwrap();
                
                    // Create a framebuffer that will be used for rendering
                    let mut framebuffer = Framebuffer::new(pipeline);
                    framebuffer.bind(|mut bound| {
                        // Very funny
                        bound.viewport(vek::Extent2::broadcast(size));
                    
                        Uniforms::new(shader.program(), pipeline, |mut uniforms| {
                            // Set the only uniform that doesn't change; the hdr map
                            uniforms.set_texture2d("hdr_map", &hdr);

                            // Render the cube 6 times with the appropriate shader and render target
                            for (i, view) in view_matrices.into_iter().enumerate() {
                                // Each time we render, we change the target texture
                                bound.clear(FramebufferClearBits::all());
                                let id = self.raw.as_ref().unwrap().name;
                                bound.set_target_unchecked(gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32, id, gl::COLOR_ATTACHMENT0);
                                
                                // Update the matrix and render the cube
                                let matrix = perspective * view;
                                uniforms.set_mat44f32("matrix", &matrix);
                            
                                // Render the cube
                                render(mesh); 
                            }
                        });
                    });     
                },
            }

            // Mipmaps and filters
            generate_mipmaps(gl::TEXTURE_CUBE_MAP, &Self::PARAMS);
            generate_filters(gl::TEXTURE_CUBE_MAP, &Self::PARAMS);
            gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32); 
        }
    }
}

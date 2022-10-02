use assets::{Asset, Assets};

use super::{
    Extent, Filter, ImageTexel, MipMapDescriptor, MipMapSetting,
    MultiLayerTexture, Region, Sampling, Texel, Texture, Texture2D, TextureImportSettings,
    TextureMode, Wrap, RGB,
};
use crate::{
    buffer::BufferMode,
    context::{Context, ToGlName, ToGlTarget},
    display::{Display, FaceCullMode, PrimitiveMode, RasterSettings, Viewport},
    mesh::{Mesh, MeshImportSettings},
    painter::{MultilayerIntoTarget, Painter},
    prelude::CubeMapConvolutionMode,
    shader::{FragmentStage, Processor, ShaderCompiler, VertexStage, Shader},
};
use std::{ffi::c_void, marker::PhantomData, mem::size_of, ptr::null, num::NonZeroU8, time::Instant};

// A cubemap texture that contains 6 different faces that each contain a square texture2D
// Cubemap textures are mostly used for environment mapping and reflections
// Cubemaps are internally stored in this data order:
// PositiveX, NegativeX, PositiveY, NegativeY, PositiveZ, NegativeZ
pub struct CubeMap2D<T: Texel> {
    // Internal OpenGL shit
    name: u32,

    // Main texture settings
    dimensions: vek::Extent2<u16>,
    mode: TextureMode,
    mipmap: MipMapDescriptor,

    // Boo (also sets Texture2D as !Sync and !Send)
    _phantom: PhantomData<*const T>,
}

impl<T: Texel> ToGlName for CubeMap2D<T> {
    fn name(&self) -> u32 {
        self.name
    }
}

impl<T: Texel> ToGlTarget for CubeMap2D<T> {
    fn target() -> u32 {
        gl::TEXTURE_CUBE_MAP
    }
}

impl<T: Texel> Drop for CubeMap2D<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.name);
        }
    }
}

impl<T: Texel> Texture for CubeMap2D<T> {
    type Region = (vek::Vec3<u16>, vek::Extent2<u16>);
    type T = T;

    fn dimensions(&self) -> <Self::Region as super::Region>::E {
        self.dimensions
    }

    fn mode(&self) -> super::TextureMode {
        self.mode
    }

    fn mipmap_descriptor(&self) -> &MipMapDescriptor {
        &self.mipmap
    }

    fn is_region_valid(&self, region: Self::Region) -> bool {
        let extent =
            <Self::Region as Region>::extent_from_origin(region.origin()) + region.extent();
        let dimensions = extent.is_self_smaller(self.dimensions());
        dimensions && region.origin().z < 6
    }

    unsafe fn from_raw_parts(
        name: u32,
        dimensions: <Self::Region as super::Region>::E,
        mode: TextureMode,
        mipmap: MipMapDescriptor,
    ) -> Self {
        Self {
            name,
            dimensions,
            mode,
            mipmap,
            _phantom: Default::default(),
        }
    }

    unsafe fn alloc_immutable_storage(
        name: u32,
        extent: <Self::Region as Region>::E,
        levels: u8,
        ptr: *const <Self::T as Texel>::Storage,
    ) {
        let extent = extent.as_::<i32>();
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, name);
        gl::TextureStorage2D(name, levels as i32, T::INTERNAL_FORMAT, extent.w, extent.h);

        if ptr != null() {
            for face in 0..6u32 {
                let offset = face as usize * extent.product() as usize;
                let offsetted_ptr = if !ptr.is_null() {
                    ptr.offset(offset as isize)
                } else {
                    null()
                };

                gl::TexSubImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + face,
                    0,
                    0,
                    0,
                    extent.w,
                    extent.h,
                    T::FORMAT,
                    T::TYPE,
                    offsetted_ptr as *const c_void,
                );
            }
        }
    }

    unsafe fn alloc_resizable_storage(
        name: u32,
        extent: <Self::Region as Region>::E,
        unique_level: u8,
        ptr: *const <Self::T as Texel>::Storage,
    ) {
        let extent = extent.as_::<i32>();
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, name);

        for face in 0..6u32 {
            let offset = face as usize * extent.product() as usize;
            let offsetted_ptr = if !ptr.is_null() {
                ptr.offset(offset as isize)
            } else {
                null()
            };

            gl::TexImage2D(
                gl::TEXTURE_CUBE_MAP_POSITIVE_X + face,
                unique_level as i32,
                T::INTERNAL_FORMAT as i32,
                extent.w,
                extent.h,
                0,
                T::FORMAT,
                T::TYPE,
                offsetted_ptr as *const c_void,
            );
        }
    }

    unsafe fn update_subregion(
        name: u32,
        region: Self::Region,
        level: u8,
        ptr: *const <Self::T as Texel>::Storage,
    ) {
        let origin = region.origin().as_::<i32>();
        let extent = region.extent().as_::<i32>();
        let face = origin.z as u32;
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, name);
        gl::TexSubImage2D(
            gl::TEXTURE_CUBE_MAP_POSITIVE_X + face,
            level as i32,
            origin.x,
            origin.y,
            extent.w,
            extent.h,
            T::FORMAT,
            T::TYPE,
            ptr as *const c_void,
        );
    }

    unsafe fn splat_subregion(
        name: u32,
        region: Self::Region,
        level: u8,
        ptr: *const <Self::T as Texel>::Storage,
    ) {
        let origin = region.origin().as_::<i32>();
        let extent = region.extent().as_::<i32>();
        gl::ClearTexSubImage(
            name,
            level as i32,
            origin.x,
            origin.y,
            origin.z,
            extent.w,
            extent.h,
            1,
            T::FORMAT,
            T::TYPE,
            ptr as *const c_void,
        );
    }

    unsafe fn splat(name: u32, level: u8, ptr: *const <Self::T as Texel>::Storage) {
        gl::ClearTexImage(name, level as i32, T::FORMAT, T::TYPE, ptr as *const c_void);
    }

    unsafe fn read_subregion(
        name: u32,
        region: Self::Region,
        level: u8,
        ptr: *mut <Self::T as Texel>::Storage,
        texels: u32,
    ) {
        let origin = region.origin().as_::<i32>();
        let extent = region.extent().as_::<i32>();
        let size = texels as u32 * T::bytes();
        gl::GetTextureSubImage(
            name,
            level as i32,
            origin.x,
            origin.y,
            origin.z,
            extent.w,
            extent.h,
            1,
            T::FORMAT,
            T::TYPE,
            size as i32,
            ptr as *mut c_void,
        );
    }

    unsafe fn read(name: u32, level: u8, ptr: *mut <Self::T as Texel>::Storage, texels: u32) {
        let size = texels as u32 * T::bytes();
        gl::GetTextureImage(
            name,
            level as i32,
            T::FORMAT,
            T::TYPE,
            size as i32,
            ptr as *mut c_void,
        )
    }

    unsafe fn copy_subregion_from(
        name: u32,
        other_name: u32,
        level: u8,
        other_level: u8,
        region: Self::Region,
        offset: <Self::Region as Region>::O,
    ) {
        let origin = region.origin().as_::<i32>();
        let extent = region.extent().as_::<i32>();
        let offset = offset.as_::<i32>();

        gl::CopyImageSubData(
            other_name,
            gl::TEXTURE_CUBE_MAP,
            other_level as i32,
            origin.x,
            origin.y,
            origin.z,
            name,
            gl::TEXTURE_CUBE_MAP,
            level as i32,
            offset.x,
            offset.y,
            offset.z,
            extent.w,
            extent.h,
            1,
        );
    }
}

impl<T: Texel> MultiLayerTexture for CubeMap2D<T> {
    fn is_layer_valid(&self, layer: u16) -> bool {
        layer < 6
    }
}

// This is a global resource that will help the user to convolute / transform equirectangular cubemaps to a proper CubeMap with convolution (possibly)
pub struct CubeMapConvolutor2D {
    cube: Mesh,
    shaders: [Shader; 3],
    painter: Painter<RGB<f32>, (), ()>,
    raster_settings: RasterSettings,
    view_matrices: [vek::Mat4<f32>; 6],
    proj_matrix: vek::Mat4<f32>
}

impl CubeMapConvolutor2D {
    // Create the cubemap convolutor 2D
    pub(crate) fn new(ctx: &mut Context, assets: &mut Assets) -> Self {
        // Convert the eqilateral texture to a cubemap texture
        let proj_matrix = vek::Mat4::perspective_fov_rh_no(90.0f32.to_radians(), 1.0, 1.0, 0.02, 20.0);
        use vek::Mat4;
        use vek::Vec3;

        // View matrices for the 6 different faces
        let view_matrices: [Mat4<f32>; 6] = [
            Mat4::look_at_rh(Vec3::zero(), Vec3::unit_x(), -Vec3::unit_y()), // Right
            Mat4::look_at_rh(Vec3::zero(), -Vec3::unit_x(), -Vec3::unit_y()), // Left
            Mat4::look_at_rh(Vec3::zero(), Vec3::unit_y(), Vec3::unit_z()),  // Top
            Mat4::look_at_rh(Vec3::zero(), -Vec3::unit_y(), -Vec3::unit_z()), // Bottom
            Mat4::look_at_rh(Vec3::zero(), Vec3::unit_z(), -Vec3::unit_y()), // Back
            Mat4::look_at_rh(Vec3::zero(), -Vec3::unit_z(), -Vec3::unit_y()), // Front
        ];

        // Load the shared vertex stage
        let vertex = assets
            .load::<VertexStage>("engine/shaders/projection.vrtx.glsl")
            .unwrap();
        
        // Create the 3 shaders (panorama, diffuse, specular)
        const PATHS: [&str; 3] = ["engine/shaders/hdri/panorama.frag.glsl", "engine/shaders/hdri/diffuse.frag.glsl", "engine/shaders/hdri/specular.frag.glsl"];
        let mut shaders = Vec::<Shader>::new();
        for i in 0..3 {
            let fragment = assets
                .load::<FragmentStage>(PATHS[i])
                .unwrap();            

            // Compile the shader and add it to the shaders vector
            shaders.push(ShaderCompiler::link((vertex.clone(), fragment), Processor::new(assets), ctx));
        }

        // I want debugless unwrap PWEASE RUST uwu >.<
        let shaders: [Shader; 3] = unsafe { shaders.try_into().unwrap_unchecked() };
        

        // Load in a unit cube that is inside out
        let cube = assets
            .load_with::<Mesh>(
                "engine/meshes/cube.obj",
                (
                    ctx,
                    MeshImportSettings {
                        invert_triangle_ordering: true,
                        ..Default::default()
                    },
                ),
            )
            .unwrap();
        
        // Create a rasterizer to convert the equilateral texture
        let mut painter = Painter::<RGB<f32>, (), ()>::new(ctx);

        // Create the rasterization settings
        let raster_settings = RasterSettings {
            depth_test: None,
            scissor_test: None,
            primitive: PrimitiveMode::Triangles {
                cull: Some(FaceCullMode::Back(true)),
            },
            srgb: false,
            blend: None,
        };

        Self {
            cube,
            shaders,
            painter,
            raster_settings,
            view_matrices,
            proj_matrix,
        }
    }

    // Convert an equirectangular texture2D to a simple cubemap
    pub fn from_equirectangular(&mut self, ctx: &mut Context, equirectangular: &Texture2D<RGB<f32>>, settings: TextureImportSettings) -> Option<CubeMap2D<RGB<f32>>> {
        // Make sure this texture has a 1:2 aspect ratio
        if (equirectangular.dimensions().w as f32) / (equirectangular.dimensions().h as f32) != 2.0 {
            return None;
        }

        // Create the resulting cubemap
        let dimensions = vek::Extent2::broadcast(equirectangular.dimensions().w / 4);
        let mut cubemap = CubeMap2D::<RGB<f32>>::new(ctx, settings.mode, dimensions, settings.sampling, settings.mipmaps, None)?;
        
        // Create the viewport for the painter
        let viewport = Viewport {
            origin: vek::Vec2::zero(),
            extent: dimensions,
        };

        // Render each face seperately
        for face in 0..6 {
            let miplevel = cubemap.mip_mut(0).unwrap();
            let target = miplevel.target(face as u16).unwrap();
            let mut scoped = self.painter.scope(viewport, target, (), ()).unwrap();
            let (mut rasterizer, mut uniforms) = scoped.rasterizer(ctx, &mut self.shaders[0], self.raster_settings);
            uniforms.set_sampler("panorama", equirectangular);
            uniforms.set_mat4x4("matrix", self.proj_matrix * self.view_matrices[face]);
            rasterizer.draw(&self.cube, uniforms.validate().unwrap());
        }

        // Update mipmaps moment
        cubemap.generate_mipmaps();

        Some(cubemap)
    }
    
    // Convert an equirectangular texture2D into a convoluted cubemap
    pub fn convoluted_from_requirectangular(&mut self, ctx: &mut Context, equirectangular: &Texture2D<RGB<f32>>, settings: TextureImportSettings, mode: CubeMapConvolutionMode) -> Option<CubeMap2D<RGB<f32>>> {
        let cubemap = self.from_equirectangular(ctx, equirectangular, settings)?;
        self.convoluted(ctx, &cubemap, settings, mode)
    }
    
    // Create a convoluted CubeMap2D from a default CubeMap2D
    // This assumes that the inputted cubemap is the perfect projected cubemap, and not modified in any way or sorts
    pub fn convoluted(&mut self, ctx: &mut Context, original: &CubeMap2D<RGB<f32>>, mut settings: TextureImportSettings, convolution: CubeMapConvolutionMode) -> Option<CubeMap2D<RGB<f32>>> {
        // Resolution of the cubemap depends on the convolution mode
        let dimensions = match convolution {
            CubeMapConvolutionMode::SpecularIBL => original.dimensions() / 2,
            CubeMapConvolutionMode::DiffuseIrradiance => vek::Extent2::broadcast(16),
        };

        // When using specular convolution, the original cube map MUST have mipmaps enabled
        if CubeMapConvolutionMode::SpecularIBL == convolution && original.levels() == 1 {
            return None;
        } 

        // When using specular convolutions, the output cubemap MUST have mipmaps enabled
        if CubeMapConvolutionMode::SpecularIBL == convolution {
            settings.mipmaps = MipMapSetting::Automatic;
        }
        
        // Create the outputting cubemap
        let cubemap = CubeMap2D::<RGB<f32>>::new(ctx, settings.mode, dimensions, settings.sampling, settings.mipmaps, None)?;

        // Get the shader index dependant on the mode
        let index = (convolution == CubeMapConvolutionMode::SpecularIBL) as u32 + 1;

        // Render each face seperately
        for mip in 0..cubemap.levels() {
            for face in 0..6 {
                let miplevel = cubemap.mip_mut(mip).unwrap();

                // Create the viewport for the painter for each mip level
                let viewport = Viewport {
                    origin: vek::Vec2::zero(),
                    extent: miplevel.dimensions(),
                };

                // Create a mip target and create a scoped rasterizer that will write to it
                let target = miplevel.target(face as u16).unwrap();
                let mut scoped = self.painter.scope(viewport, target, (), ()).unwrap();
                let (mut rasterizer, mut uniforms) = scoped.rasterizer(ctx, &mut self.shaders[index as usize], self.raster_settings);
                
                // Set the main uniforms for convolution
                uniforms.set_sampler("cubemap", original);
                uniforms.set_scalar("source_face_resolution", original.dimensions().w as u32);
                let roughness = mip as f32 / (cubemap.levels() as f32  - 1.0);
                uniforms.set_scalar("roughness", roughness);
                uniforms.set_mat4x4("matrix", self.proj_matrix * self.view_matrices[face]);

                // Write to the bound target
                rasterizer.draw(&self.cube, uniforms.validate().unwrap());
            }
        }

        ctx.flush();
        Some(cubemap)
    }
}

/*
impl<'a> Asset<'a> for CubeMap2D<RGB<f32>> {
    type Args = (&'a mut Context, TextureImportSettings);

    fn extensions() -> &'static [&'static str] {
        &["hdr"]
    }

    fn deserialize(data: assets::Data, args: Self::Args) -> Self {
        let (ctx, settings) = args;
        let hdr = hdrldr::load(data.bytes()).unwrap();
        let dimensions = vek::Extent2::new(hdr.width as u16, hdr.height as u16);
        // TODO: Optimize this vertical flip
        let rows = hdr.data.chunks(dimensions.w as usize);
        let flipped = rows
            .rev()
            .flat_map(|row| row.iter().cloned())
            .collect::<Vec<hdrldr::RGB>>();

        let texels = flipped
            .into_iter()
            .map(|texel| vek::Vec3::new(texel.r, texel.g, texel.b))
            .collect::<Vec<_>>();
        let sampling = Sampling {
            filter: Filter::Linear,
            wrap: Wrap::Repeat,
            ..Default::default()
        };

        // Create the equilateral texture that will then be mapped to a cubemap
        let texture = Texture2D::<RGB<f32>>::new(
            ctx,
            TextureMode::Static,
            dimensions,
            sampling,
            MipMapSetting::Disabled,
            Some(&texels),
        )
        .unwrap();



    }
}

// Convert a single panoramic texture into a HDRi cubemap
// This will also take account the convolution mode of the imported cubemap
pub fn hdri_from_panoramic(
    ctx: &mut Context,
    texture: Texture2D<RGB<f32>>,
    assets: &Assets,
    import_settings: CubeMapImportSettings,
) -> CubeMap2D<RGB<f32>> {





    // Pick the right resolution for the cubemap
    let original_dimensions = vek::Extent2::broadcast(texture.dimensions().w / 4);
    let dimensions = match import_settings.convolution {
        CubeMapConvolutionMode::Disabled => original_dimensions,
        CubeMapConvolutionMode::DiffuseIrradiance => vek::Extent2::broadcast(16),
        CubeMapConvolutionMode::SpecularIBL => vek::Extent2::broadcast(512),
    };

    // We might need to overwrite the texture mode if we are using a special convolution mode
    let mode = match import_settings.convolution {
        CubeMapConvolutionMode::SpecularIBL => TextureMode::Dynamic,
        _ => import_settings.mode,
    };

    // We might need to overwrite the mipmap settings if we are using a special convolution mode
    let mipmaps = match import_settings.convolution {
        CubeMapConvolutionMode::SpecularIBL => MipMapSetting::Manual { levels: NonZeroU8::new(4).unwrap() },
        _ => import_settings.mipmaps
    };

    // We might need to overwrite the sampling settings if we are using a special convolution mode
    let sampling = match import_settings.convolution {
        CubeMapConvolutionMode::SpecularIBL => Sampling {
            filter: Filter::Linear,
            ..Default::default()
        },
        _ => import_settings.sampling
    };
    
    // Create the cubemap, but don't initialize it with any data
    let mut cubemap = CubeMap2D::new(
        ctx,
        mode,
        dimensions,
        sampling,
        mipmaps,
        None,
    )
    .unwrap();

    
    // Iterate through all of the faces of the cubemap (for diffuse IBL)
    match import_settings.convolution {
        CubeMapConvolutionMode::DiffuseIrradiance | CubeMapConvolutionMode::Disabled => {
            for face in 0..6 {
                let miplevel = cubemap.mip_mut(0).unwrap();
                let target = miplevel.target(face as u16).unwrap();
                let mut scoped = painter.scope(viewport, target, (), ()).unwrap();
                let (mut rasterizer, mut uniforms) = scoped.rasterizer(ctx, &mut shader, raster_settings);
                uniforms.set_sampler("panorama", &texture);
                uniforms.set_mat4x4("matrix", proj * view_matrices[face]);
                rasterizer.draw(&cube, uniforms.validate().unwrap());
            }
        },
        CubeMapConvolutionMode::SpecularIBL => {
            
        },
    }

    match import_settings.convolution {
        CubeMapConvolutionMode::DiffuseIrradiance | CubeMapConvolutionMode::Disabled => match import_settings.mipmaps {
            MipMapSetting::Automatic | MipMapSetting::Manual { .. } => cubemap.generate_mipmaps(),
            _ => {},
        },
        _ => {}
    }
    
    cubemap
}
*/

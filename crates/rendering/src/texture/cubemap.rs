use assets::{Asset, Assets};

use super::{
    Extent, Filter, ImageTexel, MipMapDescriptor, MipMapSetting, MultiLayerTexture, Region,
    Sampling, Texel, Texture, Texture2D, TextureImportSettings, TextureMode, Wrap, RGB, CubeMapImportSettings,
};
use crate::{
    buffer::BufferMode,
    context::{Context, ToGlName, ToGlTarget},
    display::{Display, FaceCullMode, PrimitiveMode, RasterSettings, Viewport},
    mesh::{Mesh, MeshImportSettings},
    painter::{MultilayerIntoTarget, Painter},
    shader::{FragmentStage, Processor, ShaderCompiler, VertexStage}, prelude::CubeMapConvolutionMode,
};
use std::{ffi::c_void, marker::PhantomData, mem::size_of, ptr::null};

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

impl<'a> Asset<'a> for CubeMap2D<RGB<f32>> {
    type Args = (&'a mut Context, CubeMapImportSettings);

    fn extensions() -> &'static [&'static str] {
        &["hdr"]
    }

    // TODO: Convert this to a resource so we can convert multiple HDRis more efficiently?
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

        // Load the HDRi cubemap from the texture
        hdri_from_panoramic(ctx, texture, data.loader(), settings)
    }
}


// Convert a single panoramic texture into a HDRi cubemap
// This will also take account the convolution mode of the imported cubemap
pub fn hdri_from_panoramic(ctx: &mut Context, texture: Texture2D<RGB<f32>>, assets: &Assets, settings: CubeMapImportSettings) -> CubeMap2D<RGB<f32>> {
    // Convert the eqilateral texture to a cubemap texture
    let proj = vek::Mat4::perspective_fov_rh_no(90.0f32.to_radians(), 1.0, 1.0, 0.02, 20.0);
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

    // Create the cubemap, but don't initialize it with any data
    let dimensions = vek::Extent2::broadcast(texture.dimensions().w / 4);
    let mut cubemap = CubeMap2D::new(
        ctx,
        settings.mode,
        dimensions,
        settings.sampling,
        settings.mipmaps,
        None,
    )
    .unwrap();

    // Create the rasterization shader for the cubemap converter
    let vertex = assets
        .load::<VertexStage>("engine/shaders/projection.vrtx.glsl")
        .unwrap();
    let fragment = assets
        .load::<FragmentStage>(match settings.convolution {
            CubeMapConvolutionMode::Disabled => "engine/shaders/panorama.frag.glsl",
            CubeMapConvolutionMode::DiffuseIBL => "engine/shaders/diffuse_ibl_panorama.frag.glsl",
            CubeMapConvolutionMode::SpecularIBL => "engine/shaders/specular_ibl_panorama.frag.glsl",
        })
        .unwrap();
    let mut shader =
        ShaderCompiler::link((vertex, fragment), Processor::new(assets), ctx);

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

    // Create a rasterizer to convert the panoramic texture
    let mut painter = Painter::<RGB<f32>, (), ()>::new(ctx);
    let viewport = Viewport {
        origin: vek::Vec2::zero(),
        extent: dimensions,
    };

    // Create the rasterization settings
    let settings = RasterSettings {
        depth_test: None,
        scissor_test: None,
        primitive: PrimitiveMode::Triangles {
            cull: Some(FaceCullMode::Back(true)),
        },
        srgb: false,
        blend: None,
    };

    // Iterate through all of the faces of the cubemap
    for face in 0..6 {
        let miplevel = cubemap.mip_mut(0).unwrap();
        let target = miplevel.target(face as u16).unwrap();
        let mut scoped = painter.scope(viewport, target, (), ()).unwrap();
        let (mut rasterizer, mut uniforms) = scoped.rasterizer(ctx, &mut shader, settings);
        uniforms.set_sampler("panorama", &texture);
        uniforms.set_mat4x4("matrix", proj * view_matrices[face]);
        rasterizer.draw(&cube, uniforms.validate().unwrap());
    }

    // Update the mipmaps and return the texture
    cubemap.generate_mipmaps();
    cubemap
}
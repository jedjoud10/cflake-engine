use others::CacheManager;
use resources::ResourceManager;

use crate::{AdditionalShader, ComputeShader, Shader, SubShader, Texture, TextureDimensions, TextureWrapping, Uniform};

// Some volumetric shit
#[derive(Default)]
pub struct Volumetric {
    // The main SDF texture used for the volumetric sampling
    pub sdf_tex: usize,
    // The output, screen texture that will be rendered (PS: This texture might be downscaled from the original screen size)
    pub result_tex: usize,
    // The depth texture
    pub depth_tex: usize,
    // The compute shader ID for the SDF generator compute
    pub compute_generator_id: usize,
    // The compute shader ID
    pub compute_id: usize,
    // Check if the volumetric rendering is enabled
    pub enabled: bool,

    // Sizes
    sdf_dimension: u16,
    scale_down_factor_result: u16,
}

impl Volumetric {
    // Load the necessary compute shaders
    pub fn load_compute_shaders(&mut self, resource_manager: &mut ResourceManager, shader_cacher: &mut (CacheManager<SubShader>, CacheManager<Shader>)) {
        // Load generator compute
        self.compute_generator_id = Shader::new(
            vec!["defaults\\shaders\\volumetric\\sdf_gen.cmpt.glsl"],
            resource_manager,
            shader_cacher,
            Some(AdditionalShader::Compute(ComputeShader::default())),
            None
        )
        .2;
        // Load the volumetric compute
        self.compute_id = Shader::new(
            vec!["defaults\\shaders\\volumetric\\volumetric_screen.cmpt.glsl"],
            resource_manager,
            shader_cacher,
            Some(AdditionalShader::Compute(ComputeShader::default())),
            None
        )
        .2;
    }
    // Create the SDF texture from a simple texture, loaded into a compute shader
    // Create the textures
    pub fn create_textures(&mut self, texture_cacher: &mut CacheManager<Texture>, resolution: veclib::Vector2<u16>, sdf_dimensions: u16, scale_down_factor_result: u16) {
        self.sdf_dimension = sdf_dimensions;
        self.scale_down_factor_result = scale_down_factor_result;
        self.sdf_tex = texture_cacher.cache_unnamed_object(Texture::new()
            .set_dimensions(TextureDimensions::D3D(self.sdf_dimension, self.sdf_dimension, self.sdf_dimension))
            .set_wrapping_mode(TextureWrapping::Repeat)
            .set_idf(gl::R16F, gl::RED, gl::UNSIGNED_BYTE)
            .generate_texture(Vec::new()));
        // This texture is going to be rescaled if the window resolution changes
        self.result_tex = texture_cacher.cache_unnamed_object(Texture::new()
            .set_dimensions(TextureDimensions::D2D(resolution.x / self.scale_down_factor_result, resolution.y / self.scale_down_factor_result))
            .set_idf(gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE)
            .set_filter(crate::TextureFilter::Linear)
            .set_wrapping_mode(crate::TextureWrapping::ClampToBorder)
            .generate_texture(Vec::new()));
        // Depth texture
        self.depth_tex = texture_cacher.cache_unnamed_object(Texture::new()
            .set_dimensions(TextureDimensions::D2D(resolution.x / self.scale_down_factor_result, resolution.y / self.scale_down_factor_result))
            .set_idf(gl::R32F, gl::RED, gl::UNSIGNED_BYTE)
            .set_filter(crate::TextureFilter::Nearest)
            .set_wrapping_mode(crate::TextureWrapping::ClampToBorder)
            .generate_texture(Vec::new()));
    }
    // When the screen resolution changes
    pub fn update_texture_resolution(&mut self, resolution: veclib::Vector2<u16>, texture_cacher: &mut CacheManager<Texture>) {
        let result_texture = texture_cacher.id_get_object_mut(self.result_tex).unwrap();
        result_texture.update_size(TextureDimensions::D2D(resolution.x / self.scale_down_factor_result, resolution.y / self.scale_down_factor_result));
        let depth_texture = texture_cacher.id_get_object_mut(self.depth_tex).unwrap();
        depth_texture.update_size(TextureDimensions::D2D(resolution.x / self.scale_down_factor_result, resolution.y / self.scale_down_factor_result));
    }
    // Create the SDF texture from a compute shader complitely
    pub fn generate_sdf(&mut self, shader_cacher: &mut CacheManager<Shader>, texture_cacher: &CacheManager<Texture>) {
        let shader = shader_cacher.id_get_object_mut(self.compute_generator_id).unwrap();
        shader.use_shader();
        //shader.val("sdf_tex", Uniform::Image3D(self.sdf_tex, crate::TextureShaderAccessType::WriteOnly));
        // Actually generate the SDF
        let compute = match &mut shader.additional_shader {
            crate::AdditionalShader::None => panic!(),
            crate::AdditionalShader::Compute(x) => x,
        };
        // Run the compute
        let texture = texture_cacher.id_get_object(self.sdf_tex).unwrap();
        compute
            .run_compute((texture.get_width() as u32 / 4, texture.get_height() as u32 / 4, texture.get_depth() as u32 / 4))
            .unwrap();
        compute.get_compute_state().unwrap();
    }
    // Run the compute shader and calculate the result texture
    pub fn calculate_volumetric(
        &mut self,
        shader_cacher: &mut CacheManager<Shader>,
        projection_matrix: veclib::Matrix4x4<f32>,
        rotation: veclib::Quaternion<f32>,
        camera_position: veclib::Vector3<f32>,
        clip_planes: (f32, f32),
        texture_cacher: &CacheManager<Texture>,
    ) {
        if !self.enabled { return; }
        // Run the compute shader
        let shader = shader_cacher.id_get_object_mut(self.compute_id).unwrap();
        errors::ErrorCatcher::catch_opengl_errors().unwrap();
        // Create a custom View-Projection matrix that doesn't include the translation
        let vp_m = projection_matrix * (veclib::Matrix4x4::from_quaternion(&rotation));   
        let clip_planes = veclib::Vector2::<f32>::new(clip_planes.0, clip_planes.1);
        let vals = vec![
            ("result_tex", Uniform::Image2D(self.result_tex, crate::TextureShaderAccessType::WriteOnly)),
            ("depth_tex", Uniform::Texture2D(self.depth_tex, gl::TEXTURE1)),
            ("sdf_tex", Uniform::Texture3D(self.sdf_tex, gl::TEXTURE2)),
            ("camera_pos", Uniform::Vec3F32(camera_position)),
            ("custom_vp_matrix", Uniform::Mat44F32(vp_m)),
            ("projection_matrix", Uniform::Mat44F32(projection_matrix)),
            ("nf_planes", Uniform::Vec2F32(clip_planes)),    
        ]; 
        // Set the values
        shader.set_vals(vals, texture_cacher);        
        errors::ErrorCatcher::catch_opengl_errors().unwrap();
        // Get the actual compute shader
        let compute = match &mut shader.additional_shader {
            crate::AdditionalShader::Compute(x) => x,
            crate::AdditionalShader::None => panic!(),
        };

        // Run the actual compute shader
        let texture = texture_cacher.id_get_object(self.result_tex).unwrap();
        compute.run_compute((texture.get_width() as u32 / 16, texture.get_height() as u32 / 16, 1)).unwrap();
        compute.get_compute_state().unwrap();
        errors::ErrorCatcher::catch_opengl_errors().unwrap();
    }
    // Enable volumetric rendering
    pub fn enable(&mut self) { self.enabled = true; }
    // Disable volumetric rendering
    pub fn disable(&mut self) { self.enabled = false; }
}

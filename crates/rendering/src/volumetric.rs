use assets::AssetManager;

use crate::{AdditionalShader, ComputeShader, Shader, SubShader, Texture, TextureType, TextureWrapping};

// Some volumetric shit
#[derive(Default)]
pub struct Volumetric {
    // The main SDF texture used for the volumetric sampling
    pub sdf_tex: Texture,
    // The output, screen texture that will be rendered (PS: This texture might be downscaled from the original screen size)
    pub result_tex: Texture,
    // The depth texture
    pub depth_tex: Texture,
    // The compute shader ID for the SDF generator compute
    pub compute_generator: Shader,
    // The compute shader ID
    pub compute: Shader,
    // Check if the volumetric rendering is enabled
    pub enabled: bool,

    // Sizes
    sdf_dimension: u16,
    scale_down_factor_result: u16,
}

impl Volumetric {
    // Load the necessary compute shaders
    pub fn load_compute_shaders(&mut self, asset_manager: &mut AssetManager) {
        // Load generator compute
        self.compute_generator = Shader::new(
            vec!["defaults\\shaders\\volumetric\\sdf_gen.cmpt.glsl"],
            asset_manager,
            Some(AdditionalShader::Compute(ComputeShader::default())),
            None,
        )
        .unwrap();
        // Load the volumetric compute
        self.compute = Shader::new(
            vec!["defaults\\shaders\\volumetric\\volumetric_screen.cmpt.glsl"],
            asset_manager,
            Some(AdditionalShader::Compute(ComputeShader::default())),
            None,
        )
        .unwrap();
    }
    // Create the SDF texture from a simple texture, loaded into a compute shader
    // Create the textures
    pub fn create_textures(&mut self, resolution: veclib::Vector2<u16>, sdf_dimensions: u16, scale_down_factor_result: u16) {
        self.sdf_dimension = sdf_dimensions;
        self.scale_down_factor_result = scale_down_factor_result;
        self.sdf_tex = Texture::new()
            .set_dimensions(TextureType::Texture3D(self.sdf_dimension, self.sdf_dimension, self.sdf_dimension))
            .set_wrapping_mode(TextureWrapping::Repeat)
            .set_idf(gl::R16F, gl::RED, gl::UNSIGNED_BYTE)
            .generate_texture(Vec::new())
            .unwrap();
        // This texture is going to be rescaled if the window resolution changes
        self.result_tex = Texture::new()
            .set_dimensions(TextureType::Texture2D(
                resolution.x / self.scale_down_factor_result,
                resolution.y / self.scale_down_factor_result,
            ))
            .set_idf(gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE)
            .set_filter(crate::TextureFilter::Linear)
            .set_wrapping_mode(crate::TextureWrapping::ClampToBorder)
            .generate_texture(Vec::new())
            .unwrap();
        // Depth texture
        self.depth_tex = Texture::new()
            .set_dimensions(TextureType::Texture2D(
                resolution.x / self.scale_down_factor_result,
                resolution.y / self.scale_down_factor_result,
            ))
            .set_idf(gl::R32F, gl::RED, gl::UNSIGNED_BYTE)
            .set_filter(crate::TextureFilter::Nearest)
            .set_wrapping_mode(crate::TextureWrapping::ClampToBorder)
            .generate_texture(Vec::new())
            .unwrap();
    }
    // When the screen resolution changes
    pub fn update_texture_resolution(&mut self, resolution: veclib::Vector2<u16>) {
        self.result_tex.update_size(TextureType::Texture2D(
            resolution.x / self.scale_down_factor_result,
            resolution.y / self.scale_down_factor_result,
        ));
        self.depth_tex.update_size(TextureType::Texture2D(
            resolution.x / self.scale_down_factor_result,
            resolution.y / self.scale_down_factor_result,
        ));
    }
    // Create the SDF texture from a compute shader complitely
    pub fn generate_sdf(&mut self, asset_manager: &AssetManager) {
        self.compute_generator.use_shader();
        self.compute_generator.set_i3d("sdf_tex", &self.sdf_tex, crate::TextureShaderAccessType::WriteOnly);
        // Actually generate the SDF
        let compute = match &mut self.compute_generator.additional_shader {
            crate::AdditionalShader::None => panic!(),
            crate::AdditionalShader::Compute(x) => x,
        };
        // Run the compute
        compute
            .run_compute((
                self.sdf_tex.get_width() as u32 / 4,
                self.sdf_tex.get_height() as u32 / 4,
                self.sdf_tex.get_depth() as u32 / 4,
            ))
            .unwrap();
        compute.get_compute_state().unwrap();
    }
    // Run the compute shader and calculate the result texture
    pub fn calculate_volumetric(
        &mut self,
        projection_matrix: veclib::Matrix4x4<f32>,
        rotation: veclib::Quaternion<f32>,
        camera_position: veclib::Vector3<f32>,
        clip_planes: (f32, f32),
    ) {
        if !self.enabled {
            return;
        }
        // Run the compute shader
        let shader = &mut self.compute;
        // Create a custom View-Projection matrix that doesn't include the translation
        shader.use_shader();
        let vp_m = projection_matrix * (veclib::Matrix4x4::from_quaternion(&rotation));
        shader.set_i2d("result_tex", &self.result_tex, crate::TextureShaderAccessType::WriteOnly);
        shader.set_i2d("depth_tex", &self.depth_tex, crate::TextureShaderAccessType::WriteOnly);
        shader.set_t3d("sdf_tex", &self.sdf_tex, 2);
        shader.set_vec3f32("camera_pos", &camera_position);
        shader.set_mat44("custom_vp_matrix", &vp_m);
        shader.set_mat44("projection_matrix", &projection_matrix);
        shader.set_vec2f32("nf_planes", &veclib::Vector2::<f32>::new(clip_planes.0, clip_planes.1));
        // Get the actual compute shader
        let compute = match &mut shader.additional_shader {
            crate::AdditionalShader::Compute(x) => x,
            crate::AdditionalShader::None => panic!(),
        };

        // Run the actual compute shader
        compute
            .run_compute((self.result_tex.get_width() as u32 / 16, self.result_tex.get_height() as u32 / 16, 1))
            .unwrap();
        compute.get_compute_state().unwrap();
    }
    // Enable volumetric rendering
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    // Disable volumetric rendering
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

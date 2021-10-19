use others::CacheManager;

use crate::{Shader, Texture2D, Texture3D};

// Some volumetric shit
pub struct Volumetric {
    // The main SDF texture used for the volumetric sampling
    pub sdf_tex: Texture3D,
    // The output, screen texture that will be rendered (PS: This texture might be downscaled from the original screen size)
    pub result_tex: Texture2D,
    // The compute shader ID for the SDF generator compute
    pub compute_generator_id: usize,
    // The compute shader ID
    pub compute_id: usize,
    
}

impl Volumetric {
    // Dimensions of the SDF texture
    const SDF_DIMENSIONS: u16 = 128;
    // The scale down factor for the result texture
    const RESULT_SCALE_DOWN_FC: u16 = 4;
    // Create the SDF texture from a simple texture, loaded into a compute shader
    // Create the textures
    pub fn create_textures(&mut self, resolution: veclib::Vector3<u16>) {

        self.sdf_tex = Texture3D::new().set_dimensions(Self::SDF_DIMENSIONS, Self::SDF_DIMENSIONS, Self::SDF_DIMENSIONS).set_filter(crate::TextureFilter::Linear).set_idf(gl::RED, gl::RED, gl::UNSIGNED_BYTE).generate_texture(Vec::new());
        // This texture is going to be rescaled if the window resolution changes
        self.result_tex = Texture2D::new().set_dimensions(resolution.x / Self::RESULT_SCALE_DOWN_FC, resolution.y / Self::RESULT_SCALE_DOWN_FC).set_filter(crate::TextureFilter::Linear).set_idf(gl::RGBA, gl::RGBA, gl::UNSIGNED_BYTE).generate_texture(Vec::new());
    }
    // When the screen resolution changes
    pub fn update_texture_resolution(&mut self, resolution: veclib::Vector3<u16>) {
        self.result_tex.update_size(resolution.x / Self::RESULT_SCALE_DOWN_FC, resolution.y / Self::RESULT_SCALE_DOWN_FC);
    }
    // Create the SDF texture from a compute shader complitely
    pub fn generate_sdf(&mut self, shader_cacher: &mut CacheManager<Shader>) {
        // Set the result sdf texture and run the compute shader
        let shader = shader_cacher.id_get_object(self.compute_generator_id).unwrap();
        shader.set_i3d("sdf_tex", &self.sdf_tex, crate::TextureShaderAccessType::WriteOnly);
    }
    // Run the compute shader and calculate the result texture
    pub fn calculate_volumetric(&mut self, shader_cacher: &mut CacheManager<Shader>, projection_matrix: veclib::Matrix4x4<f32>, rotation: veclib::Quaternion<f32>, camera_position: veclib::Vector3<f32>, clip_planes: (f32, f32)) {
        // Run the compute shader
        let shader = shader_cacher.id_get_object_mut(self.compute_id).unwrap();
        shader.set_vec3f32("camera_position", &camera_position);
        shader.set_vec2f32("nf_planes", &veclib::Vector2::<f32>::new(clip_planes.0, clip_planes.1));
        // Create a custom View-Projection matrix that doesn't include the translation
        let vp_m = projection_matrix * (veclib::Matrix4x4::from_quaternion(&rotation));
        shader.set_mat44("custom_vp_matrix", &vp_m);
        shader.set_i3d("sdf_tex", &self.sdf_tex, crate::TextureShaderAccessType::WriteOnly);
        shader.set_i2d("result_tex", &self.result_tex, crate::TextureShaderAccessType::WriteOnly);

        // Get the actual compute shader
        let compute = match &mut shader.additional_shader {
            crate::AdditionalShader::None => panic!(),
            crate::AdditionalShader::Compute(x) => x,
        };
        
        // Run the actual compute shader
        compute.run_compute((self.result_tex.width as u32, self.result_tex.height as u32, 0));
    }
}
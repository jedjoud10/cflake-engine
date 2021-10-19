use others::CacheManager;

use crate::{Shader, Texture2D, Texture3D};

// Some volumetric shit
pub struct Volumetric {
    // The main SDF texture used for the volumetric sampling
    pub std_tex: Texture3D,
    // The output, screen texture that will be rendered (PS: This texture might be downscaled from the original screen size)
    pub result_tex: Texture2D,
    // The compute shader ID
    pub compute_id: usize,
}

impl Volumetric {
    // Create the SDF texture from a simple texture, loaded into a compute shader
    // Create the SDF texture from a compute shader complitely
    // Run the compute shader and calculate the result texture
    pub fn calculate_volumetric(&mut self, shader_cacher: &mut CacheManager<Shader>, projection_matrix: veclib::Matrix4x4<f32>, rotation: veclib::Quaternion<f32>, camera_position: veclib::Vector3<f32>, clip_planes: (f32, f32)) {
        // Run the compute shader
        let shader = shader_cacher.id_get_object_mut(self.compute_id).unwrap();
        shader.set_vec3f32("camera_position", &camera_position);
        shader.set_vec2f32("nf_planes", &veclib::Vector2::<f32>::new(clip_planes.0, clip_planes.1));
        // Create a custom View-Projection matrix that doesn't include the translation
        let vp_m = projection_matrix * (veclib::Matrix4x4::from_quaternion(&rotation));
        shader.set_mat44("custom_vp_matrix", &vp_m);

        // Get the actual compute shader
        let compute = match &mut shader.additional_shader {
            crate::AdditionalShader::None => panic!(),
            crate::AdditionalShader::Compute(x) => x,
        };
        
        // Run the actual compute shader
        compute.run_compute((self.result_tex.width as u32, self.result_tex.height as u32, 0));
    }
}
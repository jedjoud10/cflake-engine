use others::CacheManager;
use resources::ResourceManager;

use crate::{AdditionalShader, ComputeShader, Shader, SubShader, Texture2D, Texture3D, TextureWrapping};

// Some volumetric shit
#[derive(Default)]
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
    // Load the necessary compute shaders
    pub fn load_compute_shaders(&mut self, resource_manager: &mut ResourceManager, shader_cacher: &mut (CacheManager<SubShader>, CacheManager<Shader>)) {
        // Load generator compute
        self.compute_generator_id = Shader::new(
            vec!["defaults\\shaders\\volumetric\\sdf_gen.cmpt.glsl"],
            resource_manager,
            shader_cacher,
            Some(AdditionalShader::Compute(ComputeShader::default())),
        ).2;
        // Load the volumetric compute
        //let compute_generator = Shader::new(vec!["defaults\\shaders\\volumetric\\sdf_gen.cmpt.glsl"], resource_manager, shader_cacher, Some(AdditionalShader::Compute(ComputeShader::default()))).2;
    }    
    // The scale down factor for the result texture
    const RESULT_SCALE_DOWN_FC: u16 = 4;
    // Create the SDF texture from a simple texture, loaded into a compute shader
    // Create the textures
    pub fn create_textures(&mut self, resolution: veclib::Vector2<u16>) {
        self.sdf_tex = Texture3D::new()
            .set_dimensions(Self::SDF_DIMENSIONS as u16, Self::SDF_DIMENSIONS as u16, Self::SDF_DIMENSIONS as u16)
            .set_wrapping_mode(TextureWrapping::Repeat)
            .set_idf(gl::R16F, gl::RED, gl::FLOAT)
            .generate_texture(Vec::new());
        // This texture is going to be rescaled if the window resolution changes
        self.result_tex = Texture2D::new()
            .set_dimensions(resolution.x / Self::RESULT_SCALE_DOWN_FC, resolution.y / Self::RESULT_SCALE_DOWN_FC)
            .set_idf(gl::RGBA, gl::RGBA, gl::UNSIGNED_BYTE)
            .set_wrapping_mode(crate::TextureWrapping::ClampToBorder)
            .generate_texture(Vec::new());
    }
    // When the screen resolution changes
    pub fn update_texture_resolution(&mut self, resolution: veclib::Vector3<u16>) {
        self.result_tex
            .update_size(resolution.x / Self::RESULT_SCALE_DOWN_FC, resolution.y / Self::RESULT_SCALE_DOWN_FC);
    }
    // Create the SDF texture from a compute shader complitely
    pub fn generate_sdf(&mut self, shader_cacher: &mut CacheManager<Shader>) {
        // Set the result sdf texture and run the compute shader
        let shader = shader_cacher.id_get_object_mut(self.compute_generator_id).unwrap();
        let sphere_size = 2.0;
        shader.use_shader();
        shader.set_i3d("sdf_tex", &self.sdf_tex, crate::TextureShaderAccessType::WriteOnly);
        shader.set_f32("sphere_size", &sphere_size);
        // Dimensions of the SDF texture
        const SDF_DIMENSIONS: usize = 16;     
        fn unflatten(mut index: usize) -> veclib::Vector3<u16> {
            let z = index / (SDF_DIMENSIONS);
            index -= z * (SDF_DIMENSIONS);
            let y = index / (SDF_DIMENSIONS * SDF_DIMENSIONS);
            let x = index % (SDF_DIMENSIONS);
            return veclib::Vector3::new(x as u16, y as u16, z as u16);
        }
        // Get the index from a position
        fn flatten(position: veclib::Vector3<u16>) -> usize {
            return position.x as usize + (position.y as usize * SDF_DIMENSIONS * SDF_DIMENSIONS) + (position.z as usize * SDF_DIMENSIONS);
        }

        // Create the point grid
        let mut grid: Vec<
        let mut points: Vec<veclib::Vector3<f32>> = Vec::new(); 
        // Randomly create the points
        for i in 0..10 {

        }

        shader.set_vec3f32_array("points", points.as_slice());
        // Actually generate the SDF
        let compute = match &mut shader.additional_shader {
            crate::AdditionalShader::None => panic!(),
            crate::AdditionalShader::Compute(x) => x,
        };
        // Run the compute
        compute.run_compute((self.sdf_tex.width as u32, self.sdf_tex.height as u32, self.sdf_tex.depth as u32)).unwrap();
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
    ) {
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

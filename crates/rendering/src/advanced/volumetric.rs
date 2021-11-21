use crate::RenderTask;
use crate::pipec;
use crate::pipeline::object::*;
use crate::basics::*;
use crate::utils::*;
use assets::AssetManager;

// Some volumetric shit
#[derive(Default)]
pub struct Volumetric {
    // The main SDF texture used for the volumetric sampling
    pub sdf_tex: TextureGPUObject,
    // The output, screen texture that will be rendered (PS: This texture might be downscaled from the original screen size)
    pub result_tex: TextureGPUObject,
    // The depth texture
    pub depth_tex: TextureGPUObject,
    // The compute shader ID for the SDF generator compute
    pub compute_generator: ComputeShaderGPUObject,
    // The compute shader ID
    pub compute: ComputeShaderGPUObject,
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
        self.compute_generator = pipec::compute_shader(Shader::default()
            .load_shader(vec!["defaults\\shaders\\volumetric\\sdf_gen.cmpt.glsl"], asset_manager)
            .unwrap());
        // Load the volumetric compute
        self.compute = pipec::compute_shader(Shader::default()
            .load_shader(vec!["defaults\\shaders\\volumetric\\volumetric_screen.cmpt.glsl"], asset_manager)
            .unwrap());
    }
    // Create the SDF texture from a simple texture, loaded into a compute shader
    // Create the textures
    pub fn create_textures(&mut self, resolution: veclib::Vector2<u16>, sdf_dimensions: u16, scale_down_factor_result: u16) {
        self.sdf_dimension = sdf_dimensions;
        self.scale_down_factor_result = scale_down_factor_result;
        self.sdf_tex = pipec::texture(Texture::default()
            .set_dimensions(TextureType::Texture3D(self.sdf_dimension, self.sdf_dimension, self.sdf_dimension))
            .set_wrapping_mode(TextureWrapping::Repeat)
            .set_format(TextureFormat::R16F));
        // This texture is going to be rescaled if the window resolution changes
        self.result_tex = pipec::texture(Texture::default()
            .set_dimensions(TextureType::Texture2D(
                resolution.x / self.scale_down_factor_result,
                resolution.y / self.scale_down_factor_result,
            ))
            .set_format(TextureFormat::RGBA8R)
            .set_filter(TextureFilter::Linear)
            .set_wrapping_mode(TextureWrapping::ClampToBorder));
        // Depth texture
        self.depth_tex = pipec::texture(Texture::default()
            .set_dimensions(TextureType::Texture2D(
                resolution.x / self.scale_down_factor_result,
                resolution.y / self.scale_down_factor_result,
            ))
            .set_format(TextureFormat::R32F)
            .set_filter(TextureFilter::Nearest)
            .set_wrapping_mode(TextureWrapping::ClampToBorder));
    }
    // When the screen resolution changes
    pub fn update_texture_resolution(&mut self, resolution: veclib::Vector2<u16>) {
        pipec::task_immediate(RenderTask::TextureUpdateSize(self.result_tex, TextureType::Texture2D(
            resolution.x / self.scale_down_factor_result,
            resolution.y / self.scale_down_factor_result,
        )));
        pipec::task_immediate(RenderTask::TextureUpdateSize(self.depth_tex, TextureType::Texture2D(
            resolution.x / self.scale_down_factor_result,
            resolution.y / self.scale_down_factor_result,
        )));
    }
    // Create the SDF texture from a compute shader complitely
    pub fn generate_sdf(&mut self, _asset_manager: &AssetManager) {
        let mut group = self.compute_generator.new_uniform_group();
        group.set_i3d("sdf_tex", self.sdf_tex, TextureShaderAccessType::WriteOnly);
        // Actually generate the SDF
        // Run the compute
        self.compute.run(
            self.sdf_tex.2.get_width() / 4,
            self.sdf_tex.2.get_height() / 4,
            self.sdf_tex.2.get_depth() / 4);
        self.compute.lock_state();
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
        let mut group = self.compute.new_uniform_group();
        // Create a custom View-Projection matrix that doesn't include the translation
        let vp_m = projection_matrix * (veclib::Matrix4x4::from_quaternion(&rotation));
        group.set_i2d("result_tex", self.result_tex, TextureShaderAccessType::WriteOnly);
        group.set_i2d("depth_tex", self.depth_tex, TextureShaderAccessType::WriteOnly);
        group.set_t3d("sdf_tex", self.sdf_tex, 2);
        group.set_vec3f32("camera_pos", camera_position);
        group.set_mat44("custom_vp_matrix", vp_m);
        group.set_mat44("projection_matrix", projection_matrix);
        group.set_vec2f32("nf_planes", veclib::Vector2::<f32>::new(clip_planes.0, clip_planes.1));

        // Run the actual compute shader
        self.compute.run(self.result_tex.2.get_width() / 16, self.result_tex.2.get_height() / 16, 1);
        self.compute.lock_state();
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

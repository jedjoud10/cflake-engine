use assets::AssetManager;

use crate::{AdditionalShader, ComputeShader, Shader, Texture, TextureFilter, TextureType};

// Debugs some data about the current frame in a 64x256 texture. Could be used to graph the FPS or memory usage
#[derive(Default)]
pub struct FrameStats {
    // The used texture
    pub texture: Texture,
    // The used compute shader
    pub compute: Shader,
}

impl FrameStats {
    // Load the compute shaders and generate the default texture
    pub fn load_compute_shader(&mut self, asset_manager: &mut AssetManager) {
        self.compute = Shader::new()
            .set_additional_shader(AdditionalShader::Compute(ComputeShader::default()))
            .load_shader(vec!["defaults\\shaders\\others\\frame_stats.cmpt.glsl"], asset_manager).unwrap();
        self.texture = Texture::new()
            .set_dimensions(TextureType::Texture2D(256, 512))
            .set_filter(TextureFilter::Nearest)
            .generate_texture(Vec::new()).unwrap();
    }
    // Run the compute shader and update the texture
    pub fn update_texture(&mut self, time: &others::Time) {
        // Don't forget to use it
        self.compute.use_shader();
        self.compute.set_f32("time", &(time.seconds_since_game_start as f32));
        self.compute.set_f32("fps", &(time.fps as f32));
        self.compute.set_i2d("image_stats", &self.texture, crate::TextureShaderAccessType::ReadWrite);
        // Set the uniforms
        let compute = match &mut self.compute.additional_shader {
            AdditionalShader::None => todo!(),
            AdditionalShader::Compute(x) => x,
        };
        // Run the compute shader
        compute.run_compute((self.texture.get_width() as u32 / 8, self.texture.get_height() as u32 / 8, 1)).unwrap();
        compute.get_compute_state().unwrap();
    } 
}

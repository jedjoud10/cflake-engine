use crate::advanced::ComputeShader;
use crate::basics::AdditionalShader;
use crate::basics::Shader;
use crate::basics::*;
use crate::pipeline;
use crate::pipec;
use crate::utils::*;
use assets::AssetManager;
use ecs::Entity;
use others::SmartList;

// Debugs some data about the current frame in a 64x256 texture. Could be used to graph the FPS or memory usage
#[derive(Default)]
pub struct FrameStats {
    // The used texture
    pub texture: GPUObject,
    pub entities_texture: GPUObject,
    // The used compute shader
    pub compute: GPUObject,
}

impl FrameStats {
    // Load the compute shaders and generate the default texture
    pub fn load_compute_shader(&mut self, asset_manager: &mut AssetManager) {
        self.compute = pipec::create_shader(Shader::default()
            .set_additional_shader(AdditionalShader::Compute(ComputeShader::default()))
            .load_shader(vec!["defaults\\shaders\\others\\frame_stats.cmpt.glsl"], asset_manager)
            .unwrap());
        self.texture = pipec::create_texture(Texture::default()
            .set_dimensions(TextureType::Texture2D(256, 512))
            .set_filter(TextureFilter::Nearest)
            .generate_texture(Vec::new())
            .unwrap());
        self.entities_texture = pipec::create_texture(Texture::default()
            .set_format(TextureFormat::R16F)
            .set_dimensions(TextureType::Texture1D(512))
            .set_filter(TextureFilter::Nearest)
            .generate_texture(Vec::new())
            .unwrap());
    }
    // Run the compute shader and update the texture
    pub fn update_texture(&mut self, time: &others::Time, entities: &SmartList<Entity>) {
        // Don't forget to use it
        self.compute.use_shader();
        self.compute.set_i2d("image_stats", &self.texture, TextureShaderAccessType::ReadWrite);
        self.compute.set_f32("time", &(time.seconds_since_game_start as f32));
        self.compute.set_f32("fps", &(time.fps as f32));
        // Limit the number of entities to 131072
        let mut vec = entities.elements.iter().map(|x| x.is_some()).collect::<Vec<bool>>();
        vec.resize(512, false);
        let vec = vec.iter().map(|x| if *x { 255 } else { 0 }).collect::<Vec<u8>>();
        self.entities_texture.update_data(vec);
        self.compute.set_t1d("entities_texture", &self.entities_texture, 1);

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

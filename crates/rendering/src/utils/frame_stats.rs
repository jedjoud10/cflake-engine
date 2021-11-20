use crate::basics::Shader;
use crate::basics::*;
use crate::pipec;
use crate::pipeline::*;
use crate::utils::*;
use assets::AssetManager;
use ecs::Entity;
use others::SmartList;

// Debugs some data about the current frame in a 64x256 texture. Could be used to graph the FPS or memory usage
pub struct FrameStats {
    // The used texture
    pub texture: TextureGPUObject,
    pub entities_texture: TextureGPUObject,
    // The used compute shader
    pub compute: ComputeShaderGPUObject,
}

impl FrameStats {
    // Load the compute shaders and generate the default texture
    pub fn load_compute_shader(&mut self, asset_manager: &mut AssetManager) {
        self.compute = pipec::create_compute_shader(
            Shader::default()
                .load_shader(vec!["defaults\\shaders\\others\\frame_stats.cmpt.glsl"], asset_manager)
                .unwrap(),
        );
        self.texture = pipec::create_texture(Texture::default().set_dimensions(TextureType::Texture2D(256, 512)).set_filter(TextureFilter::Nearest));
        self.entities_texture = pipec::create_texture(
            Texture::default()
                .set_format(TextureFormat::R16F)
                .set_dimensions(TextureType::Texture1D(512))
                .set_filter(TextureFilter::Nearest),
        );
    }
    // Run the compute shader and update the texture
    pub fn update_texture(&mut self, time: &others::Time, entities: &SmartList<Entity>) {
        // Don't forget to use it
        let group = self.compute.new_excecution_group();
        group.set_i2d("image_stats", self.texture, TextureShaderAccessType::ReadWrite);
        group.set_f32("time", time.seconds_since_game_start as f32);
        group.set_f32("fps", time.fps as f32);
        // Limit the number of entities to 131072
        let mut vec = entities.elements.iter().map(|x| x.is_some()).collect::<Vec<bool>>();
        vec.resize(512, false);
        let vec = vec.iter().map(|x| if *x { 255 } else { 0 }).collect::<Vec<u8>>();
        self.entities_texture.update_data(vec);
        group.set_t1d("entities_texture", self.entities_texture, 1);

        // Run the compute shader
        let x = self.texture.1.get_width();
        let y = self.texture.1.get_height();
        self.compute.run(x / 8, y / 8, 1);
        self.compute.lock_state();
    }
}

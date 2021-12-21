use crate::basics::Shader;
use crate::basics::*;
use crate::pipec;
use crate::pipeline::*;

// Debugs some data about the current frame in a 64x256 texture. Could be used to graph the FPS or memory usage
#[derive(Default)]
pub struct FrameStats {
    // The used texture
    pub texture: TextureGPUObject,
    pub entities_texture: TextureGPUObject,
    // The used compute shader
    pub compute: ComputeShaderGPUObject,
}

impl FrameStats {
    // Load the compute shaders and generate the default texture
    pub fn load_compute_shader(&mut self) {
        self.compute = pipec::compute_shader(Shader::default().load_shader(vec!["defaults\\shaders\\others\\frame_stats.cmpt.glsl"]).unwrap());
        self.texture = pipec::texture(Texture::default().set_dimensions(TextureType::Texture2D(256, 512)).set_filter(TextureFilter::Nearest));
        self.entities_texture = pipec::texture(
            Texture::default()
                .set_format(TextureFormat::R16F)
                .set_dimensions(TextureType::Texture1D(512))
                .set_filter(TextureFilter::Nearest),
        );
    }
    // Run the compute shader and update the texture
    pub fn update_texture(&mut self, time: &others::Time, mut entities: Vec<bool>) {
        // Don't forget to use it
        let mut group = self.compute.new_uniform_group();
        group.set_i2d("image_stats", self.texture, TextureShaderAccessType::ReadWrite);
        group.set_f32("time", time.elapsed as f32);
        group.set_f32("fps", time.fps as f32);
        // Limit the number of entities to 131072
        entities.resize(512, false);
        let _vec = entities.iter().map(|x| if *x { 255 } else { 0 }).collect::<Vec<u8>>();
        //self.entities_texture.update_data(vec);
        group.set_t1d("entities_texture", self.entities_texture, 1);

        // Run the compute shader
        let x = self.texture.2.get_width();
        let y = self.texture.2.get_height();
        self.compute.run(x / 8, y / 8, 1, group);
        self.compute.lock_state();
    }
}

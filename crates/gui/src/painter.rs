use crate::buffers::Buffers;
use egui::{epaint::Mesh, ClippedMesh, Color32, FontImage, Output, Rect};
use rendering::gl;
use rendering::{
    basics::{
        shader::{Shader, ShaderInitSettings},
        texture::{Texture, TextureBuilder, TextureDimensions, TextureFilter, TextureFormat, TextureLayout, TextureWrapMode},
        uniforms::Uniforms,
    },
    pipeline::{Handle, Pipeline},
    utils::DataType,
};
use vek::Clamp;

// Painter that will draw the egui elements onto the screen
pub struct Painter {
    // Store everything we need to render the egui meshes
    pub(crate) shader: Handle<Shader>,
    pub(crate) gl_font_texture: Handle<Texture>,
    pub(crate) egui_font_texture_version: Option<u64>,
    pub(crate) buffers: Buffers,
}

impl Painter {
    // Create a new painter
    // This will be called on the render thread
    pub fn new(pipeline: &mut Pipeline) -> Self {
        // Load the GUI shader
        let shader_settings = ShaderInitSettings::default()
            .source("defaults/shaders/gui/vert.vrsh.glsl")
            .source("defaults/shaders/gui/frag.frsh.glsl");
        let shader = Shader::new(shader_settings).unwrap();
        let shader = pipeline.shaders.insert(shader);
        // Load the egui font texture
        let egui_font_texture = TextureBuilder::default()
            .filter(TextureFilter::Linear)
            .layout(TextureLayout {
                data_type: DataType::U8,
                internal_format: TextureFormat::RGBA8R,
                resizable: true,
            })
            .wrap_mode(TextureWrapMode::ClampToEdge(None))
            .mipmaps(false)
            .build();
        let egui_font_texture = pipeline.textures.insert(egui_font_texture);
        Self {
            shader,
            gl_font_texture: egui_font_texture,
            buffers: Buffers::new(pipeline),
            egui_font_texture_version: Default::default(),
        }
    }
    // Draw a single egui mesh
    fn draw_mesh(&mut self, rect: Rect, mesh: Mesh, pipeline: &mut Pipeline, pixels_per_point: f32) {
        // We already have the shader bound, so we just need to draw
        // Get the rect size so we can use the scissor test
        let clip_min = vek::Vec2::new(pixels_per_point * rect.min.x, pixels_per_point * rect.min.y);
        let clip_max = vek::Vec2::new(pixels_per_point * rect.max.x, pixels_per_point * rect.max.y);
        let clip_min = clip_min.clamped(vek::Vec2::zero(), pipeline.window.dimensions().as_());
        let clip_max = clip_max.clamped(vek::Vec2::zero(), pipeline.window.dimensions().as_());
        let clip_min: vek::Vec2<i32> = clip_min.round().as_();
        let clip_max: vek::Vec2<i32> = clip_max.round().as_();

        //scissor Y coordinate is from the bottom
        unsafe {
            gl::Scissor(
                clip_min.x,
                pipeline.window.dimensions().y as i32 - clip_max.y,
                clip_max.x - clip_min.x,
                clip_max.y - clip_min.y,
            );
        }

        // Gotta fil the buffers with new data, then we can draw
        self.buffers.fill_buffers(mesh.vertices, mesh.indices);
        self.buffers.draw();
    }
    // Upload the egui font texture and update it's OpenGL counterpart if it changed
    fn upload_egui_font_texture(&mut self, pipeline: &mut Pipeline, image: &FontImage) {
        // Only update if we need to
        if Some(image.version) == self.egui_font_texture_version {
            return;
        }
        // I hate this
        let mut bytes = Vec::<u8>::with_capacity(image.pixels.len() * 4);
        for alpha in image.pixels.iter() {
            let color = Color32::from_white_alpha(*alpha);
            bytes.push(color.r());
            bytes.push(color.g());
            bytes.push(color.b());
            bytes.push(color.a());
        }

        // Update the OpenGL version
        let gl_tex = pipeline.textures.get_mut(&self.gl_font_texture);
        if let Some(gl_tex) = gl_tex {
            let dimensions = TextureDimensions::Texture2d(vek::Vec2::new(image.width as u16, image.height as u16));
            gl_tex.set_dimensions(dimensions).unwrap();
            gl_tex.set_bytes(bytes).unwrap();
            // Don't forget to update the version
            self.egui_font_texture_version = Some(image.version);
        } else {
            self.egui_font_texture_version = None;
        }
    }
    // Draw a single frame using an egui context and a painter
    pub fn draw_gui(&mut self, pipeline: &mut Pipeline, clipped_meshes: Vec<ClippedMesh>, font_image: &FontImage, _output: Output) {
        // No need to draw if we don't have any meshes or if our shader is invalid
        if clipped_meshes.is_empty() || pipeline.shaders.get(&self.shader).is_none() {
            return;
        }
        // Le texture
        self.upload_egui_font_texture(pipeline, font_image);

        // Since all the elements use the same shader, we can simply set it once
        let shader = pipeline.shaders.get(&self.shader).unwrap();
        let mut uniforms = Uniforms::new(shader.program(), pipeline, true);
        // For now, the single texture we can draw is the font texture. We won't be able to set user textures, but that is an upcoming feature
        uniforms.set_texture("u_sampler", &self.gl_font_texture);
        drop(shader);

        // OpenGL settings
        unsafe {
            // UI is rendered after the scene is rendered, so it is fine to bind to the default framebuffer since we are going to use it to render the screen quad anyways
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::BindVertexArray(self.buffers.vao);
            gl::Enable(gl::FRAMEBUFFER_SRGB);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::DEPTH_TEST);
            gl::Enable(gl::SCISSOR_TEST);
        }

        for ClippedMesh(rect, mesh) in clipped_meshes {
            self.draw_mesh(rect, mesh, pipeline, pipeline.window.pixels_per_point() as f32);
        }

        // Reset
        unsafe {
            gl::Disable(gl::FRAMEBUFFER_SRGB);
            gl::BindVertexArray(0);
            gl::Disable(gl::BLEND);
            gl::Enable(gl::CULL_FACE);
            gl::Enable(gl::DEPTH_TEST);
            gl::Disable(gl::SCISSOR_TEST);
        }
    }
}

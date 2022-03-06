use getset::Getters;
use glutin::{event_loop::EventLoop, ContextBuilder, window::WindowBuilder, WindowedContext, dpi::LogicalSize, NotCurrent, PossiblyCurrent, GlProfile, GlRequest};
use others::Time;

use crate::{basics::{material::Material, mesh::Mesh, shader::Shader, texture::Texture}, advanced::{compute::ComputeShader, atomic::AtomicGroup, shader_storage::ShaderStorage}, utils::{Window, DEFAULT_WINDOW_SIZE}};

use super::PipelineCollection;


// Pipeline that mainly contains sets of specific objects like shaders and materials
#[derive(Getters)]
pub struct Pipeline {
    // OpenGL wrapper objects
    pub meshes: PipelineCollection<Mesh>,
    pub shaders: PipelineCollection< Shader>,
    pub compute_shaders: PipelineCollection<ComputeShader>,
    pub textures: PipelineCollection<Texture>,

    // Others
    pub materials: PipelineCollection<Material>,

    // Window
    pub window: Window,
    // Timings
    #[getset(get = "pub")]
    time: Time,
}

// Initialize glutin and the window
fn init_glutin_window<U>(el: &EventLoop<U>, title: String, vsync: bool) -> WindowedContext<PossiblyCurrent> {
    let wb = WindowBuilder::new().with_resizable(true).with_title(title).with_inner_size(LogicalSize::new(
        DEFAULT_WINDOW_SIZE.x as u32,
        DEFAULT_WINDOW_SIZE.y as u32,
    ));
    let wc = ContextBuilder::new()
        .with_double_buffer(Some(true))
        .with_vsync(vsync)
        .with_gl_profile(GlProfile::Core)
        .with_gl_debug_flag(false)
        .with_gl(GlRequest::Latest)
        .build_windowed(wb, el)
        .unwrap();
    // Make the context a current context
    let wc = unsafe { wc.make_current().unwrap() };
    let window = wc.window();
    window.set_cursor_grab(true).unwrap();
    window.set_cursor_visible(false);
    wc
}

impl Pipeline {
    // Create a new pipeline
    pub fn new<U>(el: &EventLoop<U>, title: String, vsync: bool, fullscreen: bool) -> Self {
        let context = init_glutin_window(el, title, vsync);
        Self {
            meshes: Default::default(),
            shaders: Default::default(),
            compute_shaders: Default::default(),
            textures: Default::default(),
            materials: Default::default(),
            time: Default::default(),
            window: {
                // Create a new window
                let mut window = Window {
                    dimensions: DEFAULT_WINDOW_SIZE,
                    context,
                    fullscreen,
                };
                // Kinda useless since we already know our fullscreen state but we must update the glutin window
                window.set_fullscreen(fullscreen);
                window
            },
        }
    }
    // Called at the end of the frame to ready the pipeline for the next frame
    pub fn end_frame(&mut self) {
        self.meshes.dispose_dangling();
        self.shaders.dispose_dangling();
        self.compute_shaders.dispose_dangling();
        self.textures.dispose_dangling();
        self.materials.dispose_dangling();
    }
}
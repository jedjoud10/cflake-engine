use getset::{Getters, MutGetters, Setters};
use glutin::{
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder, GlProfile, GlRequest, PossiblyCurrent, WindowedContext,
};
use others::Time;

use crate::{
    advanced::compute::ComputeShader,
    basics::{
        material::Material,
        mesh::Mesh,
        shader::Shader,
        texture::{BundledTexture2D, Texture, Texture2D},
    },
    object::PipelineElement,
    utils::{Window, DEFAULT_WINDOW_SIZE},
};

use super::{DefaultElements, Handle, PipelineCollection, PipelineSettings, RenderingCamera, SceneRenderer};

// Pipeline that mainly contains sets of specific objects like shaders and materials
#[derive(Getters, MutGetters, Setters)]
pub struct Pipeline {
    // OpenGL wrapper objects
    pub(crate) meshes: PipelineCollection<Mesh>,
    pub(crate) shaders: PipelineCollection<Shader>,
    pub(crate) compute_shaders: PipelineCollection<ComputeShader>,

    // Multiple texture types
    pub(crate) textures: PipelineCollection<Texture2D>,
    pub(crate) bundled_textures: PipelineCollection<BundledTexture2D>,

    // Others
    pub(crate) materials: PipelineCollection<Material>,

    // Window
    #[getset(get = "pub", get_mut = "pub")]
    window: Window,
    // Timings
    #[getset(get = "pub")]
    time: Time,
    // Settings
    #[getset(get = "pub")]
    settings: PipelineSettings,
    #[getset(get = "pub")]
    defaults: DefaultElements,
    #[getset(get = "pub", get_mut = "pub")]
    camera: RenderingCamera,
}

// Initialize glutin and the window
fn init_glutin_window<U>(el: &EventLoop<U>, title: String, vsync: bool) -> WindowedContext<PossiblyCurrent> {
    let wb = WindowBuilder::new()
        .with_resizable(true)
        .with_title(title)
        .with_inner_size(LogicalSize::new(DEFAULT_WINDOW_SIZE.x as u32, DEFAULT_WINDOW_SIZE.y as u32));
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

// Initialize OpenGL
fn init_opengl(context: &WindowedContext<PossiblyCurrent>) {
    unsafe {
        gl::load_with(|x| context.get_proc_address(x));

        // Check if the gl viewport is ok
        if !gl::Viewport::is_loaded() {
            panic!()
        }

        gl::Viewport(0, 0, DEFAULT_WINDOW_SIZE.x as i32, DEFAULT_WINDOW_SIZE.y as i32);
        SceneRenderer::init_opengl();
    }
}

// Create a new pipeline and a linked scene renderer
pub fn new<U>(el: &EventLoop<U>, title: String, vsync: bool, fullscreen: bool, settings: PipelineSettings) -> (Pipeline, SceneRenderer) {
    let context = init_glutin_window(el, title, vsync);
    // Initialize OpenGL
    init_opengl(&context);
    let mut pipeline = Pipeline {
        meshes: Default::default(),
        shaders: Default::default(),
        compute_shaders: Default::default(),
        textures: Default::default(),
        bundled_textures: Default::default(),
        materials: Default::default(),
        time: Default::default(),
        camera: Default::default(),
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
        settings,
        defaults: DefaultElements::default(),
    };

    // Magic
    let defaults = DefaultElements::new(&mut pipeline);
    pipeline.defaults = defaults;

    // Create new scene renderer
    let scene_renderer = unsafe { SceneRenderer::new(&mut pipeline) };
    (pipeline, scene_renderer)
}

impl Pipeline {
    // Post-init event
    pub fn post_init(&mut self) {
        unsafe {
            gl::Finish();
        }
    }
    // Called at the start of the frame so we can clear buffers if we need to
    pub fn start_frame(&mut self, renderer: &mut SceneRenderer) {
        unsafe {
            renderer.start_frame(self);
        }
    }
    // Called at the end of the frame to ready the pipeline for the next frame
    pub fn end_frame(&mut self) {
        self.meshes.dispose_dangling();
        self.shaders.dispose_dangling();
        self.compute_shaders.dispose_dangling();
        self.textures.dispose_dangling();
        self.materials.dispose_dangling();

        // Swap the back and front buffers, so we can show the screen something
        self.window.context().swap_buffers().unwrap();
    }
    // Handle window events
    pub fn handle_window_event(&mut self, renderer: &mut SceneRenderer, event: WindowEvent, control_flow: &mut ControlFlow) {
        match event {
            WindowEvent::Resized(size) => {
                self.window.dimensions = vek::Vec2::new(size.width as u16, size.height as u16);
                renderer.resize(self)
            }
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    }
}

impl Pipeline {
    // Insert
    pub fn insert<Element: PipelineElement>(&mut self, obj: Element) -> Handle<Element> {
        obj.add(self)
    }
    // Get, get mut
    pub fn get<Element: PipelineElement>(&self, handle: &Handle<Element>) -> Option<&Element> {
        Element::find(self, handle)
    }
    pub fn get_mut<Element: PipelineElement>(&mut self, handle: &Handle<Element>) -> Option<&mut Element> {
        Element::find_mut(self, handle)
    }
}

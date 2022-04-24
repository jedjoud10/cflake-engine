use std::cell::RefCell;

use getset::{CopyGetters, Getters, MutGetters};
use glutin::{
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    Api, ContextBuilder, GlProfile, GlRequest, PossiblyCurrent, WindowedContext,
};

use crate::{
    object::Object,
    utils::{Window, DEFAULT_WINDOW_SIZE},
};

use super::{DefaultElements, Handle, PipelineSettings, PipelineStorage, RenderingCamera, SceneRenderer, SceneRenderStats};

// Pipeline that mainly contains sets of specific objects like shaders and materials
#[derive(Getters, CopyGetters, MutGetters)]
pub struct Pipeline {
    // Contains all the objects
    storage: PipelineStorage,

    // Window
    #[getset(get = "pub", get_mut = "pub")]
    window: Window,

    // Timings
    #[getset(get_copy = "pub")]
    delta: f32,
    #[getset(get_copy = "pub")]
    elapsed: f32,

    // Settings
    #[getset(get = "pub")]
    settings: PipelineSettings,
    #[getset(get = "pub")]
    defaults: DefaultElements,
    #[getset(get = "pub", get_mut = "pub")]
    camera: RenderingCamera,

    // Stats
    #[getset(get = "pub")]
    stats: RefCell<SceneRenderStats>
}

// Initialize glutin and the window
fn init_glutin_window<U>(el: &EventLoop<U>, title: String, vsync: bool) -> WindowedContext<PossiblyCurrent> {
    let wb = WindowBuilder::new()
        .with_resizable(true)
        .with_title(title)
        .with_inner_size(LogicalSize::new(DEFAULT_WINDOW_SIZE.w as u32, DEFAULT_WINDOW_SIZE.h as u32));
    let wc = ContextBuilder::new()
        .with_double_buffer(Some(true))
        .with_vsync(vsync)
        .with_gl_profile(GlProfile::Core)
        .with_gl_debug_flag(false)
        .with_gl(GlRequest::Specific(Api::OpenGl, (4, 6)))
        .build_windowed(wb, el)
        .unwrap();
    // Make the context a current context
    let wc = unsafe { wc.make_current().unwrap() };
    let _window = wc.window();
    //window.set_cursor_grab(true).unwrap();
    //window.set_cursor_visible(false);
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

        gl::Viewport(0, 0, DEFAULT_WINDOW_SIZE.w as i32, DEFAULT_WINDOW_SIZE.h as i32);
        SceneRenderer::init_opengl();
    }
}

// Create a new pipeline and a linked scene renderer
pub fn new<U>(el: &EventLoop<U>, title: String, vsync: bool, fullscreen: bool, settings: PipelineSettings) -> (Pipeline, SceneRenderer) {
    let context = init_glutin_window(el, title, vsync);
    // Initialize OpenGL
    init_opengl(&context);
    let mut pipeline = Pipeline {
        storage: PipelineStorage::default(),
        delta: Default::default(),
        elapsed: Default::default(),
        camera: Default::default(),
        window: {
            // Create a new window
            let mut window = Window {
                changed: true,
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
        stats: Default::default(),
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
    pub fn start_frame(&mut self, renderer: &mut SceneRenderer, delta: f32, elapsed: f32) {
        self.delta = delta;
        self.elapsed = elapsed;
        renderer.start_frame(self);
    }
    // Called at the end of the frame to ready the pipeline for the next frame
    pub fn end_frame(&mut self) {
        self.storage.cleanse();

        // Swap the back and front buffers, so we can actually see something
        self.window.context().swap_buffers().unwrap();
    }
    // Handle window events
    pub fn handle_window_event(&mut self, renderer: &mut SceneRenderer, event: &WindowEvent, control_flow: &mut ControlFlow) {
        match event {
            WindowEvent::Resized(size) => {
                self.window.dimensions = vek::Extent2::new(size.width.max(1), size.height.max(1));
                self.window.changed = true;
                renderer.resize(self)
            }
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    }
}

// Methods to add and remove components from the pipeline storage
impl Pipeline {
    // Add a new object
    pub fn insert<U: Object>(&mut self, mut object: U) -> Handle<U> {
        // Susus mogus?
        object.init(self);

        self.storage.insert(object)
    }
    // Get an object immutably
    pub fn get<U: Object>(&self, handle: &Handle<U>) -> Option<&U> {
        self.storage.get(handle)
    }
    // Get an object mutably
    pub fn get_mut<U: Object>(&mut self, handle: &Handle<U>) -> Option<&mut U> {
        self.storage.get_mut(handle)
    }
}

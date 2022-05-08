use std::{
    ops::{Index, IndexMut},
};


use glutin::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder, ContextBuilder, GlProfile, GlRequest};



use crate::{Cached, Handle, PipelineStorage};

// Create a new window and a valid OpenGL context
pub fn init(el: &EventLoop<()>) -> (crate::Window, crate::Context) {
    // Build a valid window
    let wb = WindowBuilder::new().with_resizable(true).with_inner_size(LogicalSize::new(1920u32, 1080));

    // Build a valid Glutin context
    let wc = ContextBuilder::new()
        .with_double_buffer(Some(true))
        .with_gl_profile(GlProfile::Core)
        .with_gl_debug_flag(false)
        .with_gl(GlRequest::Latest)
        .build_windowed(wb, el)
        .unwrap();

    // Split the context wrapper into the window and the raw OpenGL context
    let (context, window) = unsafe { wc.make_current().unwrap().split() };

    // Initialize OpenGL
    gl::load_with(|x| context.get_proc_address(x));

    // Construct a tuple using a pipeline window and the new pipeline context
    (crate::Window::new(window), crate::Context::new(context))
}

// The renderer pipeline that will contain every object using handles
pub struct Pipeline {
    // Window & Context
    window: crate::Window,
    context: crate::Context,

    // Pipeline storage for object caching
    storage: PipelineStorage,
}

impl Pipeline {
    // Create a new pipeline
    pub fn new(el: &EventLoop<()>) -> Self {
        // Initialize the OpenGL context and the window
        let (window, context) = init(el);

        Self {
            window,
            context,
            storage: Default::default(),
        }
    }

    // Called at the start of every frame
    pub fn begin(&mut self) {}

    // Called at the end of every frame
    pub fn end(&mut self) {
        self.storage.cleanse();
    }
}

// State fetchers
impl Pipeline {
    // Get the OpenGL context, since we're going to need it for creating objects
    pub fn context(&self) -> crate::Context {
        self.context.clone()
    }

    // Get the current rendering window immutably
    pub fn window(&self) -> &crate::Window {
        &self.window
    }

    // Get the current rendering window mutably
    pub fn window_mut(&mut self) -> &mut crate::Window {
        &mut self.window
    }
}

// Storage access functions
impl Pipeline {
    // Insert an object into the pipeline
    pub fn insert<T: Cached>(&mut self, object: T) -> Handle<T> {
        self.storage.insert(object)
    }

    // Get an object immutably
    pub fn get<T: Cached>(&self, handle: &Handle<T>) -> Option<&T> {
        self.storage.get(handle)
    }

    // Get an object mutably
    pub fn get_mut<T: Cached>(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        self.storage.get_mut(handle)
    }
}

impl<T: Cached> Index<Handle<T>> for Pipeline {
    type Output = T;

    fn index(&self, index: Handle<T>) -> &Self::Output {
        self.get(&index).unwrap()
    }
}

impl<T: Cached> IndexMut<Handle<T>> for Pipeline {
    fn index_mut(&mut self, index: Handle<T>) -> &mut Self::Output {
        self.get_mut(&index).unwrap()
    }
}

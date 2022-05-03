use std::{cell::Cell, thread::ThreadId, marker::PhantomData};
use glutin::{ PossiblyCurrent, ContextWrapper, window::{Window, WindowBuilder}, event_loop::EventLoop, dpi::LogicalSize, ContextBuilder, GlProfile, GlRequest, Api};
use crate::utils::DEFAULT_WINDOW_SIZE;

// Marker context handler that contains practically nothing, but that is shareable
pub struct OpenGL {
    // We add this so we cannot send/share this context between threads
    phantom: PhantomData<*const ()>,
}

// Create a Context that contains the underlying Glutin, Window, and OpenGL context
pub struct Context {
    // Marker handle
    pub gl: OpenGL,

    // Actual glutin context wrapper
    pub ctx: ContextWrapper<glutin::PossiblyCurrent, ()>,
}

// Given a glutin event loop, construct a context and a pipeline window
fn init<U>(el: &EventLoop<U>, title: String, vsync: bool) -> (crate::utils::Window, Context) {
    // Build a valid window
    let wb = WindowBuilder::new()
        .with_resizable(true)
        .with_title(title)
        .with_inner_size(LogicalSize::new(DEFAULT_WINDOW_SIZE.w as u32, DEFAULT_WINDOW_SIZE.h as u32));

    // Build a valid Glutin context
    let wc = ContextBuilder::new()
        .with_double_buffer(Some(true))
        .with_vsync(vsync)
        .with_gl_profile(GlProfile::Core)
        .with_gl_debug_flag(false)
        .with_gl(GlRequest::Specific(Api::OpenGl, (4, 6)))
        .build_windowed(wb, el)
        .unwrap();

    // Make the context a current context, and then split it
    let (ctx, window) = unsafe {
        let current = wc.make_current().unwrap();
        // Le splitting into a window and a raw context
        current.split()
    };

    // OpenGL context wrapper 
    let context = Context {
        gl: OpenGL { phantom: Default::default() },
        ctx,
    };    
    
    // Construct a tuple using a pipeline window and the new pipeline context
    (crate::utils::Window {
        changed: true,
        dimensions: DEFAULT_WINDOW_SIZE,
        context,
        fullscreen: false,
    }, context)
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

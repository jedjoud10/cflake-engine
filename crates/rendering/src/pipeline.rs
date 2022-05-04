
use glutin::{window::WindowBuilder, dpi::LogicalSize, ContextBuilder, GlProfile, GlRequest, event_loop::EventLoop};

// Create a new window and a valid OpenGL context
pub fn init(el: &EventLoop<()>) -> (crate::Window, crate::Context) {
    // Build a valid window
    let wb = WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(LogicalSize::new(1920u32, 1080));

    // Build a valid Glutin context
    let wc = ContextBuilder::new()
        .with_double_buffer(Some(true))
        .with_gl_profile(GlProfile::Core)
        .with_gl_debug_flag(false)
        .with_gl(GlRequest::Latest)
        .build_windowed(wb, el)
        .unwrap();

    // Split the context wrapper into the window and the raw OpenGL context
    let (context, window) = unsafe {
        wc.make_current().unwrap().split()
    };

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
}
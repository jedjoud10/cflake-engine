use super::{Context, Window};
use glutin::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{Fullscreen, WindowBuilder},
    ContextBuilder, GlProfile, GlRequest,
};

// FPS cap setting that limits the number of redraws per second
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FrameRateLimit {
    VSync,
    Limited(u32),
}

// These settings will be needed to be able to create the underlying graphics pipeline
// This is a resource that will be automatically added onto the world by our main app
pub struct GraphicsSetupSettings {
    pub title: String,
    pub size: vek::Extent2<u16>,
    pub fullscreen: bool,
    pub limit: Option<FrameRateLimit>,
}

// Create a new graphics pipeline using the approriate settings
// This will be called internally by the system, and the system will simply pass the settings from the app
pub(crate) fn new<T>(settings: GraphicsSetupSettings, el: &EventLoop<T>) -> (Window, Context) {
    // Decompose le struct
    let GraphicsSetupSettings {
        title,
        size,
        fullscreen,
        limit,
    } = settings;

    // In case we want to use fullscreen, get the main monitor's video mode
    let fullscreen = if fullscreen {
        el.primary_monitor()
            .and_then(|monitor| monitor.video_modes().next())
            .map(Fullscreen::Exclusive)
    } else {
        None
    };

    // Build a valid window
    let wb = WindowBuilder::new()
        .with_resizable(true)
        .with_title(title)
        .with_fullscreen(fullscreen)
        .with_inner_size(LogicalSize::new(size.w as u32, size.h as u32));

    // Build a valid Glutin context
    let wc = ContextBuilder::new()
        .with_double_buffer(Some(true))
        .with_gl_profile(GlProfile::Core)
        .with_gl_debug_flag(true)
        .with_vsync(limit == Some(FrameRateLimit::VSync))
        .with_gl(GlRequest::Latest)
        .build_windowed(wb, el)
        .unwrap();

    // Split the context wrapper into the window and the raw OpenGL context
    let (context, window) = unsafe { wc.make_current().unwrap().split() };
    // Initialize OpenGL
    gl::load_with(|x| context.get_proc_address(x));

    // To create a window, we must have a context
    let mut ctx = super::Context::new(context);
    let window = super::Window::new(&mut ctx, window);

    // Print the default init message
    println!("OpenGL Version: {}", ctx.gl_version());
    println!("GLSL Version: {}", ctx.glsl_version());
    println!("GPU Name: {}", window.device().name());
    println!("GPU Vendor: {}", window.device().vendor());

    // Return the new graphics pipeline
    (window, ctx)
}

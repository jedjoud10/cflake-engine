use glutin::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{Fullscreen, WindowBuilder},
    ContextBuilder, GlProfile, GlRequest,
};
use world::Resource;



// A wrapper that contains both the context and the underlying device
// It's like the whole render pipeline
#[derive(Resource)]
pub struct Graphics(pub super::Device, pub super::Context);

// These settings will be needed to be able to create the underlying graphics pipeline
// This is a resource that will be automatically added onto the world by our main app
pub struct GraphicsSetupSettings {
    pub title: String,
    pub size: vek::Extent2<u16>,
    pub fullscreen: bool,
    pub vsync: bool,
}

impl Graphics {
    // Create a new graphics pipeline using the approriate settings
    // This will be called internally by the system, and the system will simply pass the settings from the app
    pub(crate) fn new<T>(settings: GraphicsSetupSettings, el: &EventLoop<T>) -> Self {
        // Decompose le struct
        let GraphicsSetupSettings {
            title,
            size,
            fullscreen,
            vsync,
        } = settings;

        // Build a valid window
        let wb = WindowBuilder::new()
            .with_resizable(true)
            .with_title(title)
            .with_fullscreen(fullscreen.then_some(Fullscreen::Borderless(None)))
            .with_inner_size(LogicalSize::new(size.w as u32, size.h as u32));

        // Build a valid Glutin context
        let wc = ContextBuilder::new()
            .with_double_buffer(None)
            .with_gl_profile(GlProfile::Core)
            .with_gl_debug_flag(true)
            .with_vsync(vsync)
            .with_gl(GlRequest::Latest)
            .build_windowed(wb, el)
            .unwrap();

        // Split the context wrapper into the window and the raw OpenGL context
        let (context, window) = unsafe { wc.make_current().unwrap().split() };

        // Initialize OpenGL
        gl::load_with(|x| context.get_proc_address(x));

        // To create a device, we must have a context
        let mut ctx = super::Context::new(context);
        let device = super::Device::new(&mut ctx, window);

        // Print the default init message
        println!("OpenGL Version: {}", ctx.gl_version());
        println!("GLSL Version: {}", ctx.glsl_version());
        println!("GPU Name: {}", device.name());
        println!("GPU Vendor: {}", device.vendor());

        // Return the new graphics pipeline
        Self(device, ctx)
    }
}

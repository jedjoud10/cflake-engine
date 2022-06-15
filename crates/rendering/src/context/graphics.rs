use glutin::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{Fullscreen, WindowBuilder},
    ContextBuilder, GlProfile, GlRequest,
};
use world::resources::Resource;

// A wrapper that contains both the context and the underlying device
// It's like the whole render pipeline
#[derive(Resource)]
#[Locked]
pub struct Graphics(pub super::Device, pub super::Context);

impl Graphics {
    // Create some new graphics given a glutin event loop
    pub fn new<T>(
        el: EventLoop<T>,
        title: String,
        size: vek::Extent2<u16>,
        fullscreen: bool,
        vsync: bool,
    ) -> (EventLoop<T>, Self) {
        // Build a valid window
        let wb = WindowBuilder::new()
            .with_resizable(true)
            .with_title(title)
            .with_fullscreen(fullscreen.then_some(Fullscreen::Borderless(None)))
            .with_inner_size(LogicalSize::new(size.w as u32, size.h as u32));

        // Build a valid Glutin context
        let wc = ContextBuilder::new()
            .with_double_buffer(Some(true))
            .with_gl_profile(GlProfile::Core)
            .with_gl_debug_flag(true)
            .with_vsync(vsync)
            .with_gl(GlRequest::Latest)
            .build_windowed(wb, &el)
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

        // Return the event loop along side the graphics pipeline
        (el, Self(device, ctx))
    }
}

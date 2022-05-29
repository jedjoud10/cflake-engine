use glutin::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder, ContextBuilder, GlProfile, GlRequest};

// A wrapper that contains both the context and the underlying device
// It's like the whole render pipeline
pub struct Graphics(pub super::Device, pub super::Context);

impl Graphics {
    // Create some new graphics given a glutin event loop
    pub fn new<T>(el: EventLoop<T>) -> (EventLoop<T>, Self) {
        // Build a valid window
        let wb = WindowBuilder::new().with_resizable(true).with_inner_size(LogicalSize::new(1920u32, 1080));

        // Build a valid Glutin context
        let wc = ContextBuilder::new()
            .with_double_buffer(Some(true))
            .with_gl_profile(GlProfile::Core)
            .with_gl_debug_flag(false)
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

        // Return the event loop along side the graphics pipeline
        (el, Self(device, ctx))
    }

    // Apply all the changes that we commited to the main framebuffer, and swap the front and back buffers
    pub fn draw(&mut self) {
        self.1.raw().swap_buffers();
    }
}

use glutin::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder, ContextBuilder, GlProfile, GlRequest};


// A wrapper that contains both the context and the underlying device
// It's like the whole render pipeline
pub struct Graphics {
    // Both are stored in a tuple since we must access both of them at the same time
    shared: (super::Device, super::Context),
}

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

        // Construct a tuple using a pipeline window and the new pipeline context
        (el, Self {
            shared: (super::Device::new(window), super::Context::new(context))
        })
    }

    // Get the underlying tuple immutably
    pub fn get(&self) -> (&super::Device, &super::Context) {
        (&self.shared.0, &self.shared.1)
    }
    
    // Get the underlying tuple mutably
    pub fn get_mut(&mut self) -> (&mut super::Device, &mut super::Context) {
        (&mut self.shared.0, &mut self.shared.1)
    }
}


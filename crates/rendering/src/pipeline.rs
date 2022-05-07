use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    sync::{
        atomic::{AtomicU16, AtomicU32, Ordering},
        Arc,
    },
};

use ahash::AHashMap;
use glutin::{dpi::LogicalSize, event_loop::EventLoop, window::WindowBuilder, ContextBuilder, GlProfile, GlRequest};
use parking_lot::RwLock;
use slotmap::SlotMap;

use crate::PipelineStorage;

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

    // Called at the start of every frame
    pub fn begin(&mut self) {

    }
    // Called at the end of every frame
    pub fn end(&mut self) {

    }
}
use crate::{object::OpenGLObjectNotInitialized, pipeline::Pipeline};
use std::marker::PhantomData;

// A wrapper around an OpenGL fence, so we can check wether or not some GPU command has finished executing
// TODO: Actually make this asynchronous

pub type MaybeGlTracker<'a, Caller, GlResult> = Result<GlTracker<'a, Caller, GlResult>, OpenGLObjectNotInitialized>;

pub struct GlTracker<'a, Caller, GlResult> {
    // An OpenGL fence object
    //fence: *const gl::types::__GLsync,
    _phantom: PhantomData<Caller>,
    result: &'a GlResult,
}

impl<'a, Caller, GlResult> GlTracker<'a, Caller, GlResult> {
    // Create a GlTracker, and call the start function
    pub(crate) fn new<F: FnOnce() -> &'a GlResult>(start: F) -> Self {
        unsafe {
            // Flush
            gl::Flush()
        }
        let me = Self {
            _phantom: PhantomData::default(),
            result: start(),
        };
        unsafe {
            // TODO: Fix this
            gl::Finish();
        }
        me
    }
    // Check wether the corresponding fence object has completed
    // TODO: Fucking shit isn't async
    pub fn completed(&self) -> &GlResult {
        self.result
    }
}

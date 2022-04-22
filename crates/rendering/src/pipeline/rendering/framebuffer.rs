use std::marker::PhantomData;

use getset::Getters;
use gl::types::GLuint;

use crate::{
    basics::texture::{Texture, Texture2D},
    pipeline::{Handle, Pipeline},
};
use bitflags::bitflags;

// Framebuffer clear bits
bitflags! {
    pub struct FramebufferClearBits: u32 {
        const COLOR = gl::COLOR_BUFFER_BIT;
        const DEPTH = gl::DEPTH_BUFFER_BIT;
    }
}

// Client state tracking for the currently bound framebuffer (and for it's resolution as well)
static mut CURRENTLY_BOUND_FRAMEBUFFER: u32 = 0;
static mut CURRENT_VIEWPORT_SIZE: vek::Extent2<u32> = vek::Extent2::new(0, 0);


// A framebuffer that can be drawn or cleared
#[derive(Getters)]
#[getset(get = "pub")]
pub struct Framebuffer {
    // OpenGL name of the frame buffer
    id: GLuint,

    // This is here to prevent the user from sending the frame buffer to another thread
    _phantom: PhantomData<*const ()>,

    // Current viewport size of the framebuffer
    viewport: vek::Extent2<u32>,
}

// Bind a raw opengl framebuffer, and set the viewport size
unsafe fn bind(id: GLuint, viewport: vek::Extent2<u32>) {
    gl::BindFramebuffer(gl::FRAMEBUFFER, id);
    gl::Viewport(0, 0, viewport.w as i32, viewport.h as i32);
    CURRENTLY_BOUND_FRAMEBUFFER = id;
    CURRENT_VIEWPORT_SIZE = viewport;
}


// Currently bound framebuffer that we can modify
pub struct BoundFramebuffer<'a> {
    fb: &'a mut Framebuffer
}

impl<'a> BoundFramebuffer<'a> {
    // Clear the framebuffer
    pub fn clear(&mut self, bits: FramebufferClearBits) {
        unsafe {
            gl::Clear(bits.bits);
        }
    }
    // Disable the draw / read color buffers
    pub fn disable_draw_read_buffers(&mut self) {
        unsafe {
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);
        }
    }
    // Bind textures to the framebuffer
    pub fn bind_textures(&mut self, pipeline: &Pipeline, textures_and_attachements: &[(Handle<Texture2D>, u32)]) {
        // Keep track of the color attachements, since we will need to set them using glDrawBuffers
        let mut color_attachements = Vec::new();

        for (handle, attachement) in textures_and_attachements.iter() {
            let texture = pipeline.get(handle).unwrap();
            unsafe {
                gl::NamedFramebufferTexture(self.fb.id, *attachement, texture.name().unwrap(), 0);
            }

            // Check if this attachement is a color attachement
            if let 36064..=36079 = attachement {
                // Valid color attachement
                color_attachements.push(*attachement);
            }
        }

        unsafe {
            // Set color attachements
            gl::NamedFramebufferDrawBuffers(self.fb.id, color_attachements.len() as i32, color_attachements.as_ptr() as *const u32);
        }

        // Always check state
        let status = unsafe { gl::CheckFramebufferStatus(gl::FRAMEBUFFER) };
        if status != gl::FRAMEBUFFER_COMPLETE {
            panic!("Framebuffer state is incomplete. Error while binding textures: '{:#x}'", status);
        }
    }
    // Set the viewport size of the framebuffer
    pub fn viewport(&mut self, size: vek::Extent2<u32>) {
        unsafe {
            gl::Viewport(0, 0, size.w as i32, size.h as i32);
            self.fb.viewport = size;
            CURRENT_VIEWPORT_SIZE = size;
        }
    }
}

impl Framebuffer {
    // Create a new framebuffer
    pub fn new(_pipeline: &Pipeline) -> Self {
        // Generate a new framebuffer
        let mut id = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut id);
        }

        Self {
            id,
            viewport: vek::Extent2::one(),
            _phantom: Default::default(),
        }
    }
    // Create a new framebuffer from the raw OpenGL id
    pub unsafe fn from_raw_parts(_pipeline: &Pipeline, id: GLuint, viewport: vek::Extent2<u32>) -> Self {
        Self {
            id,
            viewport,
            _phantom: Default::default(),
        }
    }
    // Bind the framebuffer and run the given closure while it is bound
    // This binds the shader nonetheless
    unsafe fn bind_unchecked(&mut self, closure: impl FnOnce(BoundFramebuffer)) {
        // Check the currently bound frame buffer
        let old_fb_bound = CURRENTLY_BOUND_FRAMEBUFFER;
        let old_size = CURRENT_VIEWPORT_SIZE;

        // Bind the framebuffer as the current frame buffer
        bind(self.id, self.viewport);

        // Create a bound frame buffer that we can mutate
        let bound = BoundFramebuffer {
            fb: self,
        };

        // Mutate the frame buffer
        closure(bound);
        
        // And reset to the old framebuffer (also reset the viewport size)
        bind(old_fb_bound, old_size);
    }

    // Bind the shader only if needed
    pub fn bind(&mut self, mut closure: impl FnOnce(BoundFramebuffer)) {
        unsafe {
            self.bind_unchecked(closure);
        }
    }
}

impl Drop for Framebuffer {
    // Dispose of the frame buffer
    fn drop(&mut self) {
        unsafe { gl::DeleteFramebuffers(1, &self.id) }
    }
}
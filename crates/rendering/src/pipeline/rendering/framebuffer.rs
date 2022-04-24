use std::marker::PhantomData;

use getset::Getters;
use gl::types::GLuint;

use crate::{
    basics::texture::{Texture, Texture2D},
    pipeline::{Handle, Pipeline}, object::ObjectSealed,
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
static mut BOUND: bool = false;


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
    pub fn bind_textures<Tex: ObjectSealed + Texture>(&mut self, pipeline: &Pipeline, textures_and_attachments: &[(Handle<Tex>, GLuint)]) -> Option<()> {
        // Keep track of the color attachments, since we will need to set them using glDrawBuffers
        let mut color_attachments = Vec::new();

        for (handle, attachment) in textures_and_attachments.iter() {
            let texture = pipeline.get(handle)?;
            unsafe {
                gl::NamedFramebufferTexture(self.fb.id, *attachment, texture.name()?, 0);
            }

            // Check if this attachment is a color attachment
            if let 36064..=36079 = attachment {
                // Valid color attachment
                color_attachments.push(*attachment);
            }
        }

        unsafe {
            // Set color attachments
            gl::NamedFramebufferDrawBuffers(self.fb.id, color_attachments.len() as i32, color_attachments.as_ptr() as *const u32);
        }

        // Always check state
        let status = unsafe { gl::CheckFramebufferStatus(gl::FRAMEBUFFER) };
        if status != gl::FRAMEBUFFER_COMPLETE {
            panic!("Framebuffer state is incomplete. Error while binding textures: '{:#x}'", status);
        }

        Some(())
    }
    // Set a target texture directly, using it's OpenGL name and a target
    pub unsafe fn set_target_unchecked(&mut self, target: GLuint, id: GLuint, attachment: GLuint) {
        // Set the attachment normally
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, attachment, target, target, 0);
        
        // Special functionality for cases where the attachment is a color attachment
        if let 36064..=36079 = attachment {
            gl::NamedFramebufferDrawBuffers(self.fb.id, 1, &attachment);
        }
    }
    // Set a single texture as the target texture when drawing
    pub fn set_target<Tex: ObjectSealed + Texture>(&mut self, pipeline: &Pipeline, texture: Handle<Tex>, attachment: GLuint) -> Option<()> {
        // A bit of trolling?
        let texture = pipeline.get(&texture)?;
        unsafe {
            self.set_target_unchecked(texture.target()?, texture.name()?, attachment)
        }
        Some(())
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
        // Bind the framebuffer as the current frame buffer
        bind(self.id, self.viewport);
        BOUND = true;

        // Create a bound frame buffer that we can mutate
        let bound = BoundFramebuffer {
            fb: self,
        };

        // Mutate the frame buffer
        closure(bound);
        BOUND = false;
    }

    // Bind the shader only if needed
    pub fn bind(&mut self, closure: impl FnOnce(BoundFramebuffer)) {
        unsafe {
            assert!(!BOUND, "Cannot recursively bind frambeuffers!");
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
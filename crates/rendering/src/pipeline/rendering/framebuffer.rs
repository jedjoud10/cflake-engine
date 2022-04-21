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

// Client state tracking for the currently bound framebuffer
static mut CURRENTLY_BOUND_FRAMEBUFFER: u32 = 0;

// A framebuffer that can be drawn or cleared
#[derive(Getters)]
#[getset(get = "pub")]
pub struct Framebuffer {
    pub(crate) id: GLuint,
    pub(crate) bits: FramebufferClearBits,
    pub(crate) _phantom: PhantomData<*const ()>,
}

impl Framebuffer {
    // Create a new framebuffer
    pub fn new(_pipeline: &Pipeline, bits: FramebufferClearBits) -> Self {
        // Generate a new framebuffer
        let mut id = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut id);
        }

        Self {
            id,
            bits,
            _phantom: Default::default(),
        }
    }
    // Bind textures to the frame buffer
    pub fn bind_textures(&mut self, pipeline: &Pipeline, textures_and_attachements: &[(Handle<Texture2D>, u32)]) {
        // Bind
        self.bind(|me| {
            // Keep track of the color attachements, since we will need to set them using glDrawBuffers
            let mut color_attachements = Vec::new();

            for (handle, attachement) in textures_and_attachements.iter() {
                let texture = pipeline.get(handle).unwrap();
                unsafe {
                    gl::NamedFramebufferTexture(me.id, *attachement, texture.name().unwrap(), 0);
                }

                // Check if this attachement is a color attachement
                match attachement {
                    36064..=36079 => {
                        // Valid color attachement
                        color_attachements.push(*attachement);
                    }
                    _ => { /* Not a color attachement */ }
                }
            }

            unsafe {
                // Set color attachements
                gl::NamedFramebufferDrawBuffers(me.id, color_attachements.len() as i32, color_attachements.as_ptr() as *const u32);
            }

            // Always check state
            let status = unsafe { gl::CheckFramebufferStatus(gl::FRAMEBUFFER) };
            if status != gl::FRAMEBUFFER_COMPLETE {
                panic!("Framebuffer has failed initialization! Error: '{:#x}'", status);
            }
        })
    }
    // Clear the framebuffer
    pub fn clear(&mut self) {
        unsafe {
            self.bind(|me| {
                gl::Clear(me.bits.bits);
            })
        }
    }
    // Bind the framebuffer and run the given closure while it is bound
    pub fn bind(&mut self, mut closure: impl FnMut(&mut Self)) {
        // Check the currently bound frame buffer
        let currently_bound = unsafe { CURRENTLY_BOUND_FRAMEBUFFER };
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);
            CURRENTLY_BOUND_FRAMEBUFFER = self.id;
        }
        closure(self);
        // Bind the framebuffer that *lost* it's binding, if needed
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, currently_bound);
        }
    }
}

impl Drop for Framebuffer {
    // Dispose of the frame buffer
    fn drop(&mut self) {
        unsafe { gl::DeleteFramebuffers(1, &self.id) }
    }
}

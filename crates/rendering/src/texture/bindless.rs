use std::{
    cell::Cell,
    num::NonZeroU32,
    rc::Rc,
    time::{Duration, Instant},
};

use crate::context::Context;

use super::TextureMode;

// Some unique data that will be specifically valid for bindless textures
pub struct Bindless {
    // The GPU handle for the texture
    pub(crate) handle: u64,

    // Is the handle resident (does the texutre live on the GPU)?
    pub(crate) resident: Cell<bool>,

    // The last time the bindless texture was used inside a shader
    pub(crate) last_shader_usage: Cell<Instant>,

    // How much time it takes for a texture to become non-resident
    pub(crate) timeout: Duration,
}

impl Drop for Bindless {
    fn drop(&mut self) {
        // If we drop a bindless handle, we must make it non-resident
        unsafe {
            self.resident.set(false);
            gl::MakeTextureHandleNonResidentARB(self.handle);
        }
    }
}

impl Bindless {
    // Get the bindless handle
    pub fn handle(&self) -> u64 {
        self.handle
    }

    // Get the current residency state
    pub fn is_resident(&self) -> bool {
        self.resident.get()
    }

    // Get the current time-out value
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    // Get the last time the bindless texture was used inside a shader
    pub fn last_shader_usage(&self) -> Instant {
        self.last_shader_usage.get()
    }
}

// Create a new bindless handle for a texture
pub(super) unsafe fn create_bindless(
    ctx: &mut Context,
    name: u32,
    timeout: u64,
    mode: TextureMode,
) -> Option<Rc<Bindless>> {
    (mode == TextureMode::Dynamic).then(|| {
        // Create the RC first
        let rc = Rc::new(Bindless {
            handle: gl::GetTextureHandleARB(name),
            resident: Cell::new(false),
            timeout: Duration::from_millis(timeout),
            last_shader_usage: Cell::new(Instant::now()),
        });

        // Then clone it to be able to store it within the context
        //ctx.bindless.push(rc.clone());

        // Boink
        rc
    })
}

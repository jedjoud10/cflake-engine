use std::{
    cell::Cell,
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

    // The last time the bindless texture was made resident
    pub(crate) last_residency_instant: Cell<Instant>,

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

    // Get the last time the bindless texture was made resident
    pub fn last_residency_instant(&self) -> Instant {
        self.last_residency_instant.get()
    }

    // Store the new residency state (either stored within system memory or within ram)
    pub fn set_residency(&self, resident: bool) {
        unsafe {
            if resident {
                gl::MakeTextureHandleResidentARB(self.handle);
                self.last_residency_instant.set(Instant::now());
            } else {
                gl::MakeTextureHandleNonResidentARB(self.handle);
            }
        }

        self.resident.set(resident);
    }
}

// Create a new bindless handle for a texture
pub(super) unsafe fn create_bindless(
    _ctx: &mut Context,
    name: u32,
    timeout: u64,
    mode: TextureMode,
) -> Option<Rc<Bindless>> {
    (mode == TextureMode::Dynamic).then(|| {
        Rc::new(Bindless {
            handle: gl::GetTextureHandleARB(name),
            resident: Cell::new(false),
            timeout: Duration::from_millis(timeout),
            last_residency_instant: Cell::new(Instant::now()),
        })
    })
}

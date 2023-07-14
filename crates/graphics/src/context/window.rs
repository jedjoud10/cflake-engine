use std::sync::Arc;
use thiserror::Error;

//use crate::{Normalized, RenderTarget, WindowAsTargetError, BGRA, RGBA};

// Frame rate limit of the window (can be disabled by selecting Unlimited)
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum FrameRateLimit {
    // Limit the FPS to the screen refresh rate and use VSync
    VSync,

    // Limit the FPS to a specific value
    Limited(u32),

    // There is no FPS cap
    #[default]
    Unlimited,
}

// Window setting that will tell Winit how to create the window
#[derive(Clone)]
pub struct WindowSettings {
    pub title: String,
    pub fullscreen: bool,
    pub limit: FrameRateLimit,
}

// Format of the swapchain / window presentable texture
//pub type SwapchainFormat = BGRA<Normalized<u8>>;

// A window is what we will draw to at the end of each frame
pub struct Window {
    // Raw winit window and settings
    pub(crate) settings: WindowSettings,
    pub(crate) raw: Arc<winit::window::Window>,
    pub(crate) size: vek::Extent2<u32>,
}


#[derive(Error, Debug)]
#[error("Cannot use the window as a render target since the backing texture was not acquired yet / already been presented")]
pub struct WindowAsTargetError;

impl Window {
    // Get the internal window settings
    pub fn settings(&self) -> &WindowSettings {
        &self.settings
    }

    // Get the raw winit window
    pub fn raw(&self) -> &winit::window::Window {
        &self.raw
    }

    /*
    // Get the texture render that we can render to
    pub fn as_render_target(
        &mut self,
    ) -> Result<RenderTarget<SwapchainFormat>, WindowAsTargetError> {
        self.presentable_texture_view
            .as_ref()
            .map(|view| RenderTarget {
                _phantom: std::marker::PhantomData,
                view,
            })
            .ok_or(WindowAsTargetError)
    }
    */

    // Get the current size of the window in pixels
    pub fn size(&self) -> vek::Extent2<u32> {
        self.size
    }
}

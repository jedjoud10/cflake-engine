use getset::{CopyGetters, Getters, MutGetters};
use glutin::{window::Fullscreen, PossiblyCurrent, WindowedContext};

// Get the default width and height of the starting window
pub const DEFAULT_WINDOW_SIZE: vek::Extent2<u32> = vek::Extent2::new(1280, 720);

// A window class to organize things
#[derive(Getters, CopyGetters, MutGetters)]
pub struct Window {
    #[getset(get_copy = "pub")]
    pub(crate) dimensions: vek::Extent2<u32>,
    #[getset(get = "pub", get_mut = "pub")]
    pub(crate) context: WindowedContext<PossiblyCurrent>,
    #[getset(get_copy = "pub")]
    pub(crate) fullscreen: bool,
    #[getset(get_copy = "pub")]
    pub(crate) focused: bool,
}

impl Window {
    // Enable/disable fullscreen for the window
    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        if fullscreen {
            // Enable fullscreen
            let vm = self.context.window().primary_monitor().unwrap().video_modes().next().unwrap();
            self.context.window().set_fullscreen(Some(Fullscreen::Exclusive(vm)));
        } else {
            // Disable fullscreen
            self.context.window().set_fullscreen(None);
        }
        self.fullscreen = fullscreen;
    }
    // Calculate the pixels per point
    pub fn pixels_per_point(&self) -> f64 {
        self.context.window().scale_factor()
    }
}

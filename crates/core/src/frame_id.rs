// A unique identifier for each frame
#[derive(Default, PartialEq, Eq)]
pub struct FrameID {
    pub count: u64,
}

impl FrameID {
    // Get the current global FrameID
    pub fn now() -> Self {
        FrameID { count: crate::global::timings::frame_count() }
    }
}
use std::{
    num::NonZeroU32,
    time::{Duration, Instant},
};

// Global resource that defines the time since the start of the engine and the current frame data
pub struct Time {
    // Related to delta, time, and frames
    pub(crate) delta: Duration,
    pub(crate) frame_count: u128,
    pub(crate) startup: Instant,
    pub(crate) frame_start: Instant,

    // Related to constant ticks
    pub(crate) tick_rate: u32,
    pub(crate) tick_rate_max: u32,
    pub(crate) tick_delta: Duration,
    pub(crate) tick_count: u128,
    pub(crate) local_tick_count: u32,
    pub(crate) last_tick_start: Instant,
    pub(crate) tick_interpolation: f32,
    pub(crate) accumulator: f32,
    pub(crate) ticks_to_execute: Option<NonZeroU32>,
}

impl Time {
    // Get the time it took to complete one frame
    pub fn delta(&self) -> Duration {
        self.delta
    }

    // Get the total frame count
    pub fn frame_count(&self) -> u128 {
        self.frame_count
    }

    // Get the moment we started the engine
    pub fn startup(&self) -> Instant {
        self.startup
    }

    // Calculate the elapsed time that have passed since the start of the engine
    pub fn elapsed(&self) -> Duration {
        Instant::now() - self.startup()
    }

    // Get the moment the current frame started
    pub fn frame_start(&self) -> Instant {
        self.frame_start
    }

    // Get the total tick count
    pub fn tick_count(&self) -> u128 {
        self.tick_count
    }

    // Get the local tick count
    pub fn local_tick_count(&self) -> u32 {
        self.local_tick_count
    }

    // Get the elapsed time between this tick and the last tick
    pub fn tick_delta(&self) -> Duration {
        self.tick_delta
    }

    // Represents how far we are from the last tick to the next tick (range of 0 - 1)
    pub fn tick_interpolation(&self) -> f32 {
        self.tick_interpolation
    }

    // Check how many ticks we should execute this frame
    pub fn ticks_to_execute(&self) -> Option<NonZeroU32> {
        self.ticks_to_execute
    }

    // Get the tick rate
    pub fn tick_rate(&self) -> u32 {
        self.tick_rate
    }
}

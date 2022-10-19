use std::any::Any;

// Sound samples that can be recorded / played
pub trait AudioSamples: Any + Sync + Send {
    // Check if the audio samples are stereo or mono
    fn is_mono(&self) -> bool;

    // Get the bitrate of the audio samples
    fn bitrate(&self) -> u32;
}


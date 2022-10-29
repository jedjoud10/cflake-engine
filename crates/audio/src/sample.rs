use std::any::Any;

// Sound samples descriptors contain basic information about the sound samples
pub struct SoundSamplesDescriptor {
    mono: bool,
    bitrate: u32,
    channels: u32,
}

// Sound samples that can be recorded / played
pub trait AudioSamples: Any + Sync + Send {}

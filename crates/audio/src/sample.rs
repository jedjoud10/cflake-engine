use std::{any::Any, sync::Arc, time::{Instant, Duration}};
use cpal::{traits::DeviceTrait, StreamConfig, Stream, BuildStreamError, Sample};
use parking_lot::{RwLock, Mutex};
use crate::{AudioPlayer};


// Sound samples descriptors contain basic information about the sound samples
#[derive(Debug, Clone, Copy)]
pub struct AudioSamplesDescriptor {
    // Bitrate of the audio samples in kb/s
    pub(crate) bitrate: u32,

    // Sample rate of the audio samples in hertz
    pub(crate) sample_rate: u32,

    // Number of channels in the audio samples
    pub(crate) channels: u16,
}

impl AudioSamplesDescriptor {
    // Get the bitrate of the audio samples in kb/s
    pub fn bitrate(&self) -> u32 {
        self.bitrate
    }

    // Get the sample rate of the audio samples in hertz
    pub fn sampler_rate(&self) -> u32 {
        self.sample_rate
    }
    
    // Get the number of channels in the audio samples
    pub fn channels(&self) -> u16 {
        self.channels
    }
}


// Audio input data passed to modifiers / generators / mixers
pub struct AudioContext {
    // Current time the audio's been playing
    pub(crate) time_since_start: Duration,
    pub(crate) clip_duration: Option<Duration>,

    // Current frame index from CPAL 'dst'
    pub(crate) frame_index: usize,
}

impl AudioContext {
    // Get the current time since stream creation
    pub fn time_since_creation(&self) -> Duration {
        self.time_since_start
    }
    
    // Get the total audio generator duration (None if infinite)
    pub fn clip_duration(&self) -> Option<Duration> {
        self.clip_duration
    }
    
    // Get the current frame index
    pub fn frame_index(&self) -> usize {
        self.frame_index
    }
}


// Sound samples that can be played
pub trait PlayableAudioSamples: Any + Sync + Send {
    // Descriptor related to these audio samples
    fn descriptor(&self) -> AudioSamplesDescriptor;

    // Play the audio samples to a specific listener
    fn build_output_stream(
        &self,
        listener: &AudioPlayer,
    ) -> Result<Stream, BuildStreamError>;
}

// Audio samples for T (where T is a CPAL sample)
impl<T: Sample + Send + Sync + 'static> PlayableAudioSamples for (Arc<[T]>, AudioSamplesDescriptor) {
    fn descriptor(&self) -> AudioSamplesDescriptor {
        self.1
    }

    fn build_output_stream(
        &self,
        listener: &AudioPlayer,
    ) -> Result<Stream, BuildStreamError> {
        let descriptor = self.descriptor();        
        let device = &listener.device;
        let config = listener.find_audio_stream_config(descriptor.channels, descriptor.sample_rate).unwrap();
        build_output_stream::<T>(
            self.0.clone(),
            config,
            device
        )
    }
}

// Internal function that actually builds the CPAL stream
fn build_output_stream<T: Sample + Send + Sync + 'static>(
    src: Arc<[T]>,
    config: StreamConfig,
    device: &cpal::Device
) -> Result<Stream, BuildStreamError> {
    let channels = config.channels as usize;
    let mut index = 0;

    log::debug!("Building CPAL audio stream...");
    device.build_output_stream(
        &config,
        move |dst: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Split the stream dst variable into 'frames' that contain 'channels' n number of elements
            for frame in dst.chunks_mut(channels) {
                // Stop the stream if we reached the end of the source data
                if (index+1) >= src.len() {
                    for dst_channel in frame {
                        *dst_channel = 0.0f32;
                    }
                    return;
                }

                // Magic occurs here...

                // Offset the index to continue playing the next segment
                index += channels;
            }
        },
        move |err| {
            log::error!("{}", err);
        },
    )
}
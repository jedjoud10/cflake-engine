use std::{any::Any, sync::Arc};
use cpal::{traits::DeviceTrait, StreamConfig, Stream, BuildStreamError, Sample};
use parking_lot::{RwLock, Mutex};
use crate::AudioListener;


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

// Audio settings that can be applied to audio samples to affect how they sound
// TODO: Optimize this??
pub struct AudioSamplesSettings {
    // Volume of the audio samples 
    pub(crate) volume: Arc<Mutex<f32>>,

    // Callback function that executes over each frame of audio
    pub(crate) callback: Option<Arc<dyn Fn(&mut [f32]) + Send + Sync>>,
}

// Sound samples that can be played
pub trait PlayableAudioSamples: Any + Sync + Send {
    // Descriptor related to these audio samples
    fn descriptor(&self) -> AudioSamplesDescriptor;

    // Play the audio samples to a specific listener
    fn build_output_stream(
        &self,
        listener: &AudioListener,
        settings: &AudioSamplesSettings,
    ) -> Result<Stream, BuildStreamError>;
}

// Audio samples for T (where T is a CPAL sample)
impl<T: Sample + Send + Sync + 'static> PlayableAudioSamples for (Arc<[T]>, AudioSamplesDescriptor) {
    fn descriptor(&self) -> AudioSamplesDescriptor {
        self.1
    }

    fn build_output_stream(
        &self,
        listener: &AudioListener,
        settings: &AudioSamplesSettings
    ) -> Result<Stream, BuildStreamError> {
        let descriptor = self.descriptor();        
        let device = &listener.device;
        let config = listener.find_audio_stream_config(descriptor.channels, descriptor.sample_rate).unwrap();
        build_output_stream::<T>(
            self.0.clone(),
            config,
            settings,
            device
        )
    }
}

// Internal function that actually builds the CPAL stream
fn build_output_stream<T: Sample + Send + Sync + 'static>(
    src: Arc<[T]>,
    config: StreamConfig,
    settings: &AudioSamplesSettings,
    device: &cpal::Device
) -> Result<Stream, BuildStreamError> {
    // Create and clone necessary data
    let channels = config.channels as usize;
    let volume = settings.volume.clone();
    let callback = settings.callback.clone();
    let mut index = 0;

    device.build_output_stream(
        &config,
        move |dst: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Split the stream dst variable into 'frames' that contain 'channels' n number of elements
            let volume = *volume.lock();
            for frame in dst.chunks_mut(channels) {
                // Stop the stream if we reached the end of the source data
                if (index+1) >= src.len() {
                    for dst_channel in frame {
                        *dst_channel = 0.0f32;
                    }
                    return;
                }

                // Write the destination channels using the source channel
                for (channel, channel_dst) in frame.iter_mut().enumerate() {
                    let src = src[index + channel].to_f32() * volume;

                    // Apply audio filters here...

                    *channel_dst = src;            
                }

                // Execute the callback function
                if let Some(callback) = callback.clone() {
                    callback(frame);
                }

                // Offset the index to continue playing the next segment
                index += channels;
            }
        },
        move |err| {
            log::error!("{}", err);
        },
    )
}
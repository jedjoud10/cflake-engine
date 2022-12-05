use std::{any::Any, sync::Arc};
use cpal::{traits::DeviceTrait, StreamConfig, Stream, BuildStreamError, Sample};
use parking_lot::{RwLock, Mutex};
use crate::AudioListener;


// Sound samples descriptors contain basic information about the sound samples
#[derive(Debug, Clone, Copy)]
pub struct AudioSamplesDescriptor {
    // Bitrate of the audio clip in kb/s
    pub(crate) bitrate: u32,

    // Sample rate of the audio clip in hertz
    pub(crate) sample_rate: u32,

    // Number of channels in the audio clip
    pub(crate) channels: u16,
}

// Audio settings that can be applied to audio samples to affect how they sound
pub struct AudioSamplesSettings {
    // Volume of the audio samples 
    pub(crate) volume: Arc<Mutex<f32>>,

    // Callback function that executes over each frame of audio
    pub(crate) callback: Option<Arc<dyn Fn(&mut [f32]) + Send + Sync>>,
}

// Sound samples that can be recorded / played
pub trait AudioSamples: Any + Sync + Send {
    // Descriptor related to these audio samples
    fn descriptor(&self) -> AudioSamplesDescriptor;

    // Play the audio samples to a specific CPAL device
    fn build_output_stream(
        &self,
        device: &cpal::Device,
        config: &StreamConfig,
        settings: &AudioSamplesSettings,
    ) -> Result<Stream, BuildStreamError>;
}

// Audio samples for i16
impl AudioSamples for (Arc<[i16]>, AudioSamplesDescriptor) {
    fn descriptor(&self) -> AudioSamplesDescriptor {
        self.1
    }

    fn build_output_stream(
        &self,
        device: &cpal::Device,
        config: &StreamConfig,
        settings: &AudioSamplesSettings
    ) -> Result<Stream, BuildStreamError> {        
        // Create and clone necessary data
        let src = self.0.clone();
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
}
use std::{any::Any, sync::Arc, time::{Instant, Duration}};
use cpal::{traits::DeviceTrait, StreamConfig, Stream, BuildStreamError, Sample};
use parking_lot::{RwLock, Mutex};
use crate::{AudioPlayer};

// Implemented for AudioOutputs, since they allow us to create audio streams
pub trait OutputStreamBuilder: Any + Sync + Send {
    fn build_output_stream(
        &self,
        listener: &AudioPlayer,
    ) -> Result<Stream, BuildStreamError>;
}

// Audio samples for T (where T is a CPAL sample)
impl<T: Sample + Send + Sync + 'static> OutputStreamBuilder for Arc<[T]> {
    fn build_output_stream(
        &self,
        listener: &AudioPlayer,
    ) -> Result<Stream, BuildStreamError> {
        /*
        let descriptor = self.descriptor();        
        let device = &listener.device;
        let config = listener.find_audio_stream_config(descriptor.channels, descriptor.sample_rate).unwrap();
        build_output_stream::<T>(
            self.0.clone(),
            config,
            device
        )
        */
        todo!()
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
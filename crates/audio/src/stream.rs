use crate::{AudioPlayer, Sample, AudioClip};
use cpal::{
    traits::DeviceTrait, BuildStreamError, Stream, StreamConfig,
};
use parking_lot::{Mutex, RwLock};
use std::{
    any::Any,
    marker::PhantomData,
    sync::Arc,
    time::{Duration, Instant},
};


// This will be used to create the CPAL output stream
pub trait OutputStream {
    // Create the CPAL output stream
    fn build_output_stream(
        &self,
        player: &AudioPlayer,
    ) -> Result<Stream, BuildStreamError>;
}

impl<S: Sample> OutputStream for AudioClip<S> {
    fn build_output_stream(
        &self,
        player: &AudioPlayer,
    ) -> Result<Stream, BuildStreamError> {
        let channels = self.channels();
        let sample_rate = self.sample_rate();
        let src = self.samples();
        let config = player.find_audio_stream_config(channels, sample_rate).unwrap();
        build_output_stream::<S>(config, &player.device, Box::new(move |dst, frame| {
            if frame >= src.len() {
                dst.fill(S::zero());
                return;
            }

            dst.copy_from_slice(&src[frame..][..dst.len()]);
        }))
    }
}

// This function will take in a stream config and a device and it will create the CPAL stream for us
// We can also pass it a callback function that gets called whenever we need to write some data into it
fn build_output_stream<S: Sample>(
    config: StreamConfig,
    device: &cpal::Device,
    callback: Box<dyn Fn(&mut [S], usize) + Send + Sync>,
) -> Result<Stream, BuildStreamError> {
    let channels = config.channels as usize;
    let mut index = 0;

    log::debug!("Building CPAL audio stream...");
    device.build_output_stream(
        &config,
        move |dst: &mut [S], _: &cpal::OutputCallbackInfo| {
            for frame in dst.chunks_mut(channels) {
                callback(frame, index);
                index += channels;
            }
        },
        move |err| {
            log::error!("{}", err);
        },
    )
}
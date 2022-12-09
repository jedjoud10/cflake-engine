use std::{any::Any, sync::Arc, time::{Instant, Duration}, marker::PhantomData};
use cpal::{traits::DeviceTrait, StreamConfig, Stream, BuildStreamError};
use parking_lot::{RwLock, Mutex};
use crate::{AudioPlayer, Sample, traits::AudioNode};

pub trait OutputStreamBuilder: Any + Sync + Send {
    fn build_output_stream(
        &self,
        listener: &AudioPlayer,
    ) -> Result<Stream, BuildStreamError>;
}


// Internal function that actually builds the CPAL stream
fn build_output_stream<T: Sample>(
    config: StreamConfig,
    device: &cpal::Device
) -> Result<Stream, BuildStreamError> {
    let channels = config.channels as usize;
    let mut index = 0;

    log::debug!("Building CPAL audio stream...");
    device.build_output_stream(
        &config,
        move |dst: &mut [T], c: &cpal::OutputCallbackInfo| {
            for frame in dst.chunks_mut(channels) {
                write_data(frame, channels, index);
                index += channels;
            }
        },
        move |err| {
            log::error!("{}", err);
        },
    )
}

// Write to the destination channels
fn write_data<T: Sample>(dst: &mut [T], channels: usize, index: usize) {
}
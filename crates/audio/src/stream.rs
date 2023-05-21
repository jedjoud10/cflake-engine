use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use crate::{AudioClip, AudioPlayer};
use cpal::{traits::DeviceTrait, BuildStreamError, Stream, StreamConfig};
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

// De-interleave some samples from one each other
fn deinterleave(input: &[f32], channels: usize) -> Vec<Vec<f32>> {
    let mut output = (0..channels)
        .into_iter()
        .map(|_| vec![0.0f32; input.len() / channels])
        .collect::<Vec<_>>();

    // I can already smell my CPU smoking (TODO: OPTIMIZE)
    output
        .par_iter_mut()
        .enumerate()
        .for_each(|(channel, output)| {
            output.par_iter_mut().enumerate().for_each(|(i, sample)| {
                *sample = input[i * channels + channel];
            })
        });

    output
}

// Re-interleave multiple samples into one buffer
fn interleave<B: AsRef<[f32]> + Sync>(input: &[B], channels: usize) -> Vec<f32> {
    let mut output = vec![0.0f32; input[0].as_ref().len() * channels];

    output.par_iter_mut().enumerate().for_each(|(i, sample)| {
        *sample = input[i % channels].as_ref()[i / channels];
    });

    output
}

// Convert a mono track to a stereo by duplicating the same noise multiple times
// Assumes the stereo data contains interleaved data
fn mono_to_stereo(mono: &[f32]) -> Vec<f32> {
    let mut output = vec![0.0f32; mono.len() * 2];

    output.par_iter_mut().enumerate().for_each(|(i, sample)| {
        *sample = mono[i / 2];
    });

    output
}

// Convert stereo tracks to a mono track by averaging the values
// Assumes the stereo data contains interleaved samples
fn stereo_to_mono(stereo: &[f32]) -> Vec<f32> {
    let mut output = vec![0.0f32; stereo.len() / 2];

    output.par_iter_mut().enumerate().for_each(|(i, sample)| {
        *sample = (stereo[i * 2] + stereo[i * 2 + 1]) / 2.0;
    });

    output
}

// Create a CPAL output stream for a specific audio clip
pub(super) fn build_clip_output_stream(
    clip: &AudioClip,
    player: &AudioPlayer,
) -> Result<Stream, BuildStreamError> {
    let channels = clip.channels();
    let sample_rate = clip.sample_rate();

    log::debug!(
        "Looking for audio stream config for sample rate = {sample_rate} with {channels} channels"
    );

    let mut supported_configs = player
        .supported_output_configs
        .iter()
        .cloned()
        .filter(move |config_range| {
            let channels = config_range.channels() == channels;
            let max = config_range.max_sample_rate().0 >= sample_rate;
            let min = config_range.min_sample_rate().0 <= sample_rate;
            log::debug!("Channels supported: {channels}");
            log::debug!("Max sample rate supported: {max}");
            log::debug!("Min sample rate supported: {min}");
            channels & max & min
        })
        .map(move |p| {
            p.clone()
                .with_sample_rate(cpal::SampleRate(sample_rate))
                .config()
        });

    // Pick the first config available or resample if needed
    let (config, src) = match supported_configs.next() {
        // Pick the first cxonfig available
        Some(config) => (config, clip.samples()),

        // Resample the audio clip samples
        None => {
            let configs = player.supported_output_configs[0]
                .clone()
                .with_max_sample_rate();

            // Either we have missing channels
            let src = if configs.channels() != channels {
                todo!()
            } else {
                clip.samples().clone()
            };

            // or we have the wrong number of samples
            let src = if configs.sample_rate().0 != sample_rate {
                let chunk_size_in = clip.samples().len() / (channels as usize);
                let sub_chunks = 256;

                // Create a resampler to change the sample rate
                let mut resampler = rubato::FftFixedIn::<f32>::new(
                    sample_rate as usize,
                    configs.sample_rate().0 as usize,
                    chunk_size_in,
                    sub_chunks,
                    channels as usize,
                )
                .unwrap();

                // De-interleave the data
                let wave_in = deinterleave(&clip.samples(), channels as usize);

                // Change the sample rate
                let output = rubato::Resampler::process(&mut resampler, &wave_in, None).unwrap();

                // Re-interleave the data
                let interleaved = interleave(&output, channels as usize);
                Arc::from(interleaved)
            } else {
                src.clone()
            };

            (configs.config(), src)
        }
    };

    // Create the raw CPAL stream
    build_output_raw_stream(
        config,
        &player.device,
        player.volume.clone(),
        Box::new(move |dst, frame| {
            if frame >= src.len() {
                dst.fill(0.0f32);
            }

            dst.copy_from_slice(&src[frame..][..dst.len()]);
        }),
    )
}

// This function will take in a stream config and a device and it will create the CPAL stream for us
// We can also pass it a callback function that gets called whenever we need to write some data into it
pub(super) fn build_output_raw_stream(
    config: StreamConfig,
    device: &cpal::Device,
    volume: Arc<AtomicU32>,
    callback: Box<dyn Fn(&mut [f32], usize) + Send + Sync>,
) -> Result<Stream, BuildStreamError> {
    let channels = config.channels as usize;
    let mut index = 0;

    log::debug!("Building CPAL audio stream...");

    device.build_output_stream(
        &config,
        move |dst: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Using the callback write into the stream
            for frame in dst.chunks_mut(channels) {
                callback(frame, index);
                index += channels;
            }

            // Read the volume values from the player
            let value = volume.load(Ordering::Relaxed);
            let volume = f32::from_ne_bytes(value.to_ne_bytes());

            // Affect the audio clip using the volume
            for sample in dst {
                *sample *= volume;
            }
        },
        move |err| {
            log::error!("{}", err);
        },
    )
}

use std::{sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
}, time::{Duration, Instant}};

use crate::{AudioClip, AudioListener, Source, SourceInput};
use atomic_float::AtomicF32;
use cpal::{traits::DeviceTrait, BuildStreamError, Stream, StreamConfig};
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

// De-interleave some samples from one each other
fn deinterleave(input: &[f32], channels: usize) -> Vec<Vec<f32>> {
    let mut output = (0..channels)
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
    source: Box<dyn Source>,
    player: &AudioListener,
) -> Result<Stream, BuildStreamError> {
    let channels = source.target_channels();
    let sample_rate = source.target_sample_rate();

    log::debug!(
        "Looking for audio stream config for sample rate = {sample_rate:?} with {channels:?} channels"
    );

    let mut supported_configs = player
        .supported_output_configs
        .iter()
        .cloned()
        .filter(move |config_range| {
            let channels = channels.map(|channels| config_range.channels() == channels).unwrap_or(true);
            
            let (min, max) = if let Some(sample_rate) = sample_rate {
                let max = config_range.max_sample_rate().0 >= sample_rate;
                let min = config_range.min_sample_rate().0 <= sample_rate;
                (min, max)
            } else {
                (true, true)
            };

            log::debug!("Channels supported: {channels}");
            log::debug!("Max sample rate supported: {max}");
            log::debug!("Min sample rate supported: {min}");
            channels & max & min
        })
        .map(move |p| {
            let sample_rate = sample_rate.unwrap_or(p.max_sample_rate().0);
            p
                .with_sample_rate(cpal::SampleRate(sample_rate))
                .config()
        });

    // Pick the first config available or resample if needed
    let (config, src) = match supported_configs.next() {
        // Pick the first config available
        Some(config) => (config, source),

        // Resample the audio clip samples
        None => {
            /*
            let sample_rate = sample_rate.unwrap();
            let channels = channels.unwrap();
            let source: Duration = source.duration().unwrap();

            // Buffer the whole source into a vec
            let max_samples = sample_rate as f32 * source.duration().map(|d| d.as_secs_f32()).unwrap_or_default();
            assert!(max_samples < usize::MAX as f32);
            let mut buffered = Vec::<f32>::with_capacity(max_samples as usize);

            while let Some(value) = source.sample() {
                buffered.push(value);
            }

            let configs = player.supported_output_configs[0]
                .clone()
                .with_max_sample_rate();

            // Either we have missing channels
            let src = if configs.channels() != channels {
                todo!()
            } else {
                &buffered
            };

            // or we have the wrong number of samples
            let resampled = if configs.sample_rate().0 != sample_rate {
                let chunk_size_in = buffered.len() / (channels as usize);
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
                let wave_in = deinterleave(src, channels as usize);

                // Change the sample rate
                let output = rubato::Resampler::process_into_buffer(&mut resampler, &wave_in, None).unwrap();

                // Re-interleave the data
                let interleaved = interleave(&output, channels as usize);

                // Readback the given data
                struct Readback {
                    samples: Vec<f32>,
                    channels: u16,
                    sample_rate: u32,
                }

                impl Source for Readback {
                    fn sample(&mut self, input: &SourceInput) -> Option<f32> {
                        self.samples.get(input.index * 2 + input.channel as usize).copied()
                    }

                    fn duration(&self) -> Option<std::time::Duration> {
                        todo!()
                    }

                    fn target_channels(&self) -> Option<u16> {
                        todo!()
                    }

                    fn target_sample_rate(&self) -> Option<u32> {
                        todo!()
                    }
                }

                Box::from(Readback {
                    samples: interleaved,
                    channels,
                    sample_rate,
                })
            } else {
                source
            };


            (configs.config(), resampled)
            */

            todo!()
        }
    };

    // Create the raw CPAL stream
    build_output_raw_stream(
        config,
        &player.device,
        player.volume.clone(),
        src
    )
}

// This function will take in a stream config and a device and it will create the CPAL stream for us
// We can also pass it a callback function that gets called whenever we need to write some data into it
pub(super) fn build_output_raw_stream(
    config: StreamConfig,
    device: &cpal::Device,
    volume: Arc<AtomicF32>,
    mut source: Box<impl Source + ?Sized + 'static>,
) -> Result<Stream, BuildStreamError> {
    let channels = config.channels;
    let sample_rate = config.sample_rate.0;
    let mut index = 0usize;

    log::debug!("Building CPAL audio stream...");

    device.build_output_stream(
        &config,
        move |dst: &mut [f32], _: &cpal::OutputCallbackInfo| {
            source.cache();

            // Read the volume values from the player
            let volume = volume.load(Ordering::Relaxed);
            
            // Probably could be optimized but works okay for now
            for frame in dst.chunks_mut(channels as usize) {
                let mut channel = 0;
                for sample in frame.iter_mut() {
                    let duration = index as f32 / sample_rate as f32;

                    // Create the source input used by the source trait
                    let input = SourceInput {
                        channels,
                        channel,
                        index,
                        given_sample_rate: sample_rate,
                        duration,
                    };
    
                    // Fetch the next sample
                    if let Some(value) = source.sample(&input) {
                        *sample = value * volume;
                    } else {
                        *sample = 0.0;
                    }
    
                    index += 1;
                    channel += 1;
                }
            }
        },
        move |err| {
            log::error!("{}", err);
        },
    )
}
